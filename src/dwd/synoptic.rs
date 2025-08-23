use std::{
    collections::{HashMap, hash_map::Entry},
    io::Read,
    sync::{LazyLock, RwLock},
};

use anyhow::bail;
use dwd_bufr_tables::{DWD_BUFR_TABLE_B, DWD_BUFR_TABLE_D};
use dwd_gts::GtsHeader;
use regex::Regex;
use tinybufr::{DataEvent, DataReader, DataSpec, HeaderSections, Tables, Value, XY};

use crate::dwd::{Cache, Datapoint, WeatherCondition, get_etag, needs_fetch};

const URL: &str = "https://opendata.dwd.de/weather/weather_reports/synoptic/germany/Z__C_EDZW_latest_bda01%2Csynop_bufr_GER_999999_999999__MW_XXX.bin";
const LISTING_URL: &str = "https://opendata.dwd.de/weather/weather_reports/synoptic/germany";

static HREF_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"href\s*=\s*"(Z[^"]+)""#).unwrap());

pub fn get(cache: &RwLock<Cache>, stations: &[String]) -> anyhow::Result<bool> {
    if !needs_fetch(URL, cache.read().unwrap().synop_etag.as_deref()) {
        return Ok(false);
    }

    let mut res = ureq::get(URL).call()?;
    if !res.status().is_success() {
        bail!("Failed to get synop BUFR - got status {:?}", res.status());
    }
    let etag = get_etag(&res);

    let mut datapoint = read_file_to_point(res.body_mut().as_reader(), stations)?;
    if datapoint.is_none() && last_observation_is_old(cache) {
        datapoint = try_old_reports(stations);
    }

    let mut cache = cache.write().unwrap();
    if let Some(point) = datapoint {
        cache.observation = Some(point);
    }
    cache.synop_etag = etag;

    Ok(true)
}

fn last_observation_is_old(cache: &RwLock<Cache>) -> bool {
    let Some(ts) = cache
        .read()
        .unwrap()
        .observation
        .as_ref()
        .map(|o| o.timestamp)
    else {
        return true; // no past observation
    };

    match (jiff::Timestamp::now() - ts)
        .abs()
        .total(jiff::Unit::Minute)
    {
        Ok(m) if m < 60.0 => false,
        _ => true,
    }
}

fn try_old_reports(stations: &[String]) -> Option<Datapoint> {
    let mut res = ureq::get(LISTING_URL).call().ok()?;
    if !res.status().is_success() {
        return None;
    }
    let body = res.body_mut().read_to_string().ok()?;
    for file in body.lines().rev().filter_map(|l| {
        HREF_REGEX
            .captures(l)
            .and_then(|c| c.get(1).map(|m| m.as_str()))
    }) {
        let url = format!("{LISTING_URL}/{file}");
        let Ok(mut res) = ureq::get(url)
            .call()
            .inspect_err(|e| eprintln!("Failed to fetch old report: {e}"))
        else {
            continue;
        };
        if !res.status().is_success() {
            eprintln!(
                "Fetching old report returned non-OK status - {}",
                res.status()
            );
            continue;
        }

        if let Ok(Some(datapoint)) = read_file_to_point(res.body_mut().as_reader(), stations) {
            return Some(datapoint);
        };
    }

    None
}

fn read_file_to_point(mut r: impl Read, stations: &[String]) -> anyhow::Result<Option<Datapoint>> {
    let tables = make_tables();
    let mut points = vec![None; stations.len()];

    let mut processed = HashMap::new();

    loop {
        // 8 bytes ASCII message length
        let mut gts_bytes = [0u8; 8];
        r.read_exact(&mut gts_bytes)?;
        let Some(length) = atoi::atoi::<u64>(&gts_bytes).filter(|x| *x > 0) else {
            break;
        };
        // skip two '0'
        r.read_exact(&mut gts_bytes[..2])?;

        let mut bounded = r.by_ref().take(length);
        let mut gts_header_bytes = [0; 35];
        bounded.read_exact(&mut gts_header_bytes[..31])?;
        if &gts_header_bytes[28..31] != b"\r\r\n" {
            bounded.read_exact(&mut gts_header_bytes[31..])?;
            if &gts_header_bytes[32..] != b"\r\r\n" {
                bail!("invalid GTC header end");
            }
        }
        let gts_header = GtsHeader::read(&gts_header_bytes)?;
        if length == 31 + 7 {
            let mut nil_end = [0; 7];
            bounded.read_exact(&mut nil_end)?;
            if &nil_end != b"NIL\r\r\n\x03" {
                bail!("Invalid nil message");
            }
            continue;
        }

        let my_entry = VisitedGtsMessage::new(&gts_header);
        match processed.entry((gts_header.product_id, gts_header.source)) {
            Entry::Occupied(mut occupied) => {
                if *occupied.get() >= my_entry {
                    // already seen, skip
                    std::io::copy(&mut bounded, &mut std::io::sink())?;
                    continue;
                }
                *occupied.get_mut() = my_entry;
            }
            Entry::Vacant(vacant) => {
                vacant.insert(my_entry);
            }
        }

        let mut bounded = raw_bufr(bounded, &tables, &mut points, stations)?;
        let mut footer = [0; 8];
        bounded.read_exact(&mut footer)?;
        if &footer != b"7777\r\r\n\x03" {
            bail!("invalid BUFR end or GTS end");
        }
    }

    let merged =
        points
            .into_iter()
            .filter_map(|x| x)
            .fold(None, |r: Option<Datapoint>, v| match r {
                Some(mut d) => {
                    d.merge_from(&v);
                    Some(d)
                }
                None => Some(v),
            });

    Ok(merged)
}

fn make_tables() -> Tables {
    let mut tables = Tables::default();
    for it in &DWD_BUFR_TABLE_B {
        tables.table_b.insert(it.xy, it);
    }
    for it in &DWD_BUFR_TABLE_D {
        tables.table_d.insert(it.xy, it);
    }
    tables
}

const WIGOS_LOCAL_ID: XY = XY { x: 1, y: 128 };
const DATE_SEQ: XY = XY { x: 1, y: 11 };
const TIME_SEQ: XY = XY { x: 1, y: 12 };
const TEMPERATURE: XY = XY { x: 12, y: 101 };
const CLOUD_COVER: XY = XY { x: 20, y: 10 };
const RELATIVE_HUMIDITY_A: XY = XY { x: 13, y: 3 };
const RELATIVE_HUMIDITY_B: XY = XY { x: 13, y: 9 };
const TOTAL_PRECIPITATION: XY = XY { x: 13, y: 11 };
const SIGNIFICANT_WEATHER: XY = XY { x: 20, y: 3 };
const SENSOR_HEIGHT_ABOVE_GROUND: XY = XY { x: 7, y: 32 };
const TIME_PERIOD_OR_DISPLACEMENT: XY = XY { x: 4, y: 25 };
const WIND_SPEED: XY = XY { x: 11, y: 2 };
const WIND_DIRECTION: XY = XY { x: 11, y: 1 };
const MAX_WIND_GUST_SPEED: XY = XY { x: 11, y: 41 };

const DESIRED_HEIGHT: f32 = 2.0;

pub fn raw_bufr<R: Read>(
    mut bounded: R,
    tables: &Tables,
    points: &mut Vec<Option<Datapoint>>,
    stations: &[String],
) -> anyhow::Result<R> {
    let header = HeaderSections::read(&mut bounded).unwrap();
    let data_spec =
        DataSpec::from_data_description(&header.data_description_section, &tables).unwrap();
    let mut data_reader = DataReader::new(bounded, &data_spec).unwrap();

    loop {
        let DataEvent::Data {
            value: Value::String(ident),
            ..
        } = forward_until_data(&mut data_reader, WIGOS_LOCAL_ID)?
        else {
            break;
        };
        let trimmed = ident.trim_ascii_end();

        let Some(idx) = stations.iter().position(|it| it == trimmed) else {
            continue;
        };
        if let Some(mut p) = read_datapoint(&mut data_reader)? {
            if let Some(existing) = points.get_mut(idx) {
                if let Some(existing) = existing.as_mut() {
                    p.merge_from(&existing);
                    *existing = p;
                } else {
                    *existing = Some(p)
                }
            }
        }
    }
    Ok(data_reader.into_inner())
}

fn read_datapoint(r: &mut DataReader<impl Read>) -> anyhow::Result<Option<Datapoint>> {
    if !forward_until_seq(r, DATE_SEQ)? {
        return Ok(None);
    }
    let (Some(y), Some(m), Some(d)) = (read_int(r)?, read_int(r)?, read_int(r)?) else {
        return Ok(None);
    };
    forward_until_seq(r, TIME_SEQ)?;
    let (Some(h), Some(min)) = (read_int(r)?, read_int(r)?) else {
        return Ok(None);
    };
    let timestamp = jiff::civil::Date::new(y as i16, m as i8, d as i8)?
        .to_datetime(jiff::civil::Time::new(h as i8, min as i8, 0, 0)?)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp();

    let mut point = Datapoint::from_timestamp(timestamp, true);

    let mut last_height_above_ground = None;
    let mut last_time_period = None;
    let mut repeat_level = 0;
    let mut height_of_temp = None;

    loop {
        match r.read_event() {
            Ok(DataEvent::SubsetEnd) => break,
            Ok(DataEvent::ReplicationStart { .. }) => {
                repeat_level += 1;
            }
            Ok(DataEvent::ReplicationEnd) => repeat_level -= 1,
            Ok(DataEvent::Data { xy, value, .. }) => match xy {
                SENSOR_HEIGHT_ABOVE_GROUND => {
                    if let Some(f) = value_to_float(&value) {
                        last_height_above_ground = Some(f);
                    }
                }
                TIME_PERIOD_OR_DISPLACEMENT => {
                    if let Value::Integer(i) = value {
                        last_time_period = Some(i.abs());
                    }
                }
                TEMPERATURE => {
                    let Some(value) = value_to_float(&value) else {
                        continue;
                    };
                    let value = value - 273.15;
                    let cur_height = last_height_above_ground.unwrap_or(0.0);
                    match height_of_temp {
                        None => {
                            point.temperature = Some(value);
                            height_of_temp = Some(cur_height);
                        }
                        Some(last)
                            if (last - DESIRED_HEIGHT).abs()
                                > (cur_height - DESIRED_HEIGHT).abs() =>
                        {
                            point.temperature = Some(value);
                            height_of_temp = Some(cur_height);
                        }
                        _ => (),
                    };
                }
                TOTAL_PRECIPITATION if repeat_level <= 0 => {
                    let (Some(value), Some(time_period)) =
                        (value_to_float(&value), last_time_period)
                    else {
                        continue;
                    };
                    // value is the amount in the last time period -> estimate amount in 1h
                    let value = value * 60.0 / time_period as f32;
                    point.precipitation = Some(value)
                }
                CLOUD_COVER => {
                    if let Some(value) = value_to_float(&value) {
                        point.cloud_cover = Some(value);
                    }
                }
                SIGNIFICANT_WEATHER => {
                    if let Value::Integer(i) = value {
                        point.condition = WeatherCondition::Synop(i as u16);
                    }
                }
                RELATIVE_HUMIDITY_A | RELATIVE_HUMIDITY_B => {
                    if let Some(value) = value_to_float(&value) {
                        point.relative_humidity = Some(value);
                    }
                }
                WIND_SPEED => {
                    if let Some(value) = value_to_float(&value) {
                        point.mean_wind = Some(value);
                    }
                }
                WIND_DIRECTION => {
                    if let Some(value) = value_to_float(&value) {
                        point.wind_dir = Some(value);
                    }
                }
                MAX_WIND_GUST_SPEED => {
                    if let Some(value) = value_to_float(&value) {
                        point.wind_gusts = Some(value);
                    }
                }
                _ => (),
            },
            Ok(_) => (),
            Err(_) => todo!(),
        }
    }

    Ok(Some(point))
}

fn forward_until_data(r: &mut DataReader<impl Read>, target: XY) -> anyhow::Result<DataEvent> {
    loop {
        match r.read_event() {
            Ok(e @ DataEvent::Data { xy, .. }) if xy == target => return Ok(e),
            Ok(DataEvent::Eof) => return Ok(DataEvent::Eof),
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
    }
}

fn forward_until_seq(r: &mut DataReader<impl Read>, target: XY) -> anyhow::Result<bool> {
    loop {
        match r.read_event() {
            Ok(DataEvent::SequenceStart { xy, .. }) if xy == target => return Ok(true),
            Ok(DataEvent::Eof) => return Ok(false),
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
    }
}

fn read_int(r: &mut DataReader<impl Read>) -> anyhow::Result<Option<i32>> {
    match r.read_event() {
        Ok(DataEvent::Data {
            value: Value::Integer(i),
            ..
        }) => return Ok(Some(i)),
        Ok(_) => return Ok(None),
        Err(e) => return Err(e.into()),
    }
}

fn value_to_float(v: &Value) -> Option<f32> {
    match v {
        Value::Integer(i) => Some(*i as f32),
        Value::Decimal(v, s) => Some(*v as f32 * 10f32.powi(*s as i32)),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct VisitedGtsMessage {
    day: u8,
    hour: u8,
    minute: u8,
}

impl VisitedGtsMessage {
    pub fn new(gts: &GtsHeader) -> Self {
        Self {
            day: gts.day,
            hour: gts.hour,
            minute: gts.minute,
        }
    }
}

impl PartialOrd for VisitedGtsMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.day.partial_cmp(&other.day) {
            Some(std::cmp::Ordering::Equal) => {}
            Some(std::cmp::Ordering::Less) if self.day == 1 => {
                return Some(std::cmp::Ordering::Greater);
            }
            ord => return ord,
        }
        match self.hour.partial_cmp(&other.hour) {
            Some(std::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.minute.partial_cmp(&other.minute)
    }
}

impl Ord for VisitedGtsMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.day.cmp(&other.day) {
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Less if self.day == 1 => {
                return std::cmp::Ordering::Greater;
            }
            ord => return ord,
        }
        match self.hour.cmp(&other.hour) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.minute.cmp(&other.minute)
    }
}
