use anyhow::{anyhow, bail};
use quick_xml::events::Event;
use std::{
    io::{self, BufReader, Cursor},
    ops::Deref,
    sync::RwLock,
};

use crate::{Cache, Datapoint, PoiStation, WeatherCondition, get_etag};

pub fn get(station: PoiStation, cache: &RwLock<Cache>) -> anyhow::Result<bool> {
    if !needs_fetch(station, MosmixType::L, &cache.read().unwrap())
        && !needs_fetch(station, MosmixType::S, &cache.read().unwrap())
    {
        return Ok(false);
    }
    let (rs, rl) = std::thread::scope(|s| {
        let hs = s.spawn(|| fetch(station, MosmixType::S));
        let rl = fetch(station, MosmixType::L);
        let rs = hs
            .join()
            .unwrap_or_else(|e| Err(anyhow!("Failed to join/spawn: {e:?}")));
        (rs, rl)
    });

    let mut cache = cache.write().unwrap();
    let data = match (rs, rl) {
        (Ok((s, ts)), Ok((l, tl))) => {
            cache.forecast_s_etag = ts;
            cache.forecast_l_etag = tl;
            Datapoint::merge_series_vec(s, l)
        }
        (Err(_es), Ok((l, tl))) => {
            cache.forecast_l_etag = tl;
            l
        }
        (Ok((s, ts)), Err(_el)) => {
            cache.forecast_s_etag = ts;
            s
        }
        (Err(es), Err(el)) => bail!("Failed to fetch both S ({es}) and L ({el})"),
    };
    cache.forecast = data;
    Ok(true)
}

fn fetch(station: PoiStation, ty: MosmixType) -> anyhow::Result<(Vec<Datapoint>, Option<String>)> {
    let mut res = ureq::get(ty.url(station)).call()?;
    if !res.status().is_success() {
        bail!("Failed to get station - got status {:?}", res.status());
    }
    let etag = get_etag(&res);

    let bytes = res.body_mut().with_config().limit(1 << 28).read_to_vec()?;
    let mut archive = zip::read::ZipArchive::new(Cursor::new(bytes))?;
    let file = archive.by_index(0)?;
    let data = parse(BufReader::new(file), &station.to_string())?;

    Ok((data, etag))
}

fn needs_fetch(station: PoiStation, ty: MosmixType, cache: &Cache) -> bool {
    super::needs_fetch(&ty.url(station), ty.etag(cache).as_deref())
}

fn parse(reader: impl io::BufRead, target: &str) -> anyhow::Result<Vec<Datapoint>> {
    let mut buf = Vec::new();
    let mut inner_buf = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(reader);
    reader.config_mut().trim_text(true);

    let mut datapoints = Vec::new();
    // first, get the timestamps and init the datapoints
    'outer: loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"dwd:ForecastTimeSteps" => loop {
                inner_buf.clear();
                match reader.read_event_into(&mut inner_buf)? {
                    Event::Text(text) => {
                        if let Some(timestamp) =
                            str::from_utf8(&text).ok().and_then(|s| s.parse().ok())
                        {
                            datapoints.push(Datapoint::from_timestamp(timestamp, false));
                        }
                    }
                    Event::End(e) if e.name().as_ref() == b"dwd:ForecastTimeSteps" => break 'outer,
                    Event::Eof => break,
                    _ => {}
                }
            },
            Event::Eof => break,
            _ => {}
        }
    }

    // find the station
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"kml:name" => {
                buf.clear();
                if let Event::Text(b) = reader.read_event_into(&mut buf)?
                    && b.deref() == target.as_bytes()
                {
                    read_data(&mut reader, &mut datapoints)?;
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    datapoints.sort_by_key(|it| it.timestamp);
    Ok(datapoints)
}

fn read_data(
    reader: &mut quick_xml::Reader<impl io::BufRead>,
    datapoints: &mut [Datapoint],
) -> quick_xml::Result<()> {
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"dwd:Forecast" => {
                if let Ok(Some(att)) = e.try_get_attribute("dwd:elementName") {
                    match att.value.as_ref() {
                        mosmix::SIGNIFICANT_WEATHER => {
                            parse_sig_weather_into(reader, &mut buf, datapoints)
                        }
                        mosmix::PRECIPITATION => {
                            parse_into(reader, &mut buf, datapoints, |d, v| {
                                d.precipitation = Some(v)
                            })
                        }
                        mosmix::PRECIPITATION_P => {
                            parse_into(reader, &mut buf, datapoints, |d, v| {
                                d.p_precipitation = Some(v)
                            })
                        }
                        mosmix::EFFECTIVE_CLOUD_COVER => {
                            parse_into(reader, &mut buf, datapoints, |d, v| d.cloud_cover = Some(v))
                        }
                        mosmix::TEMP_2M => parse_into(reader, &mut buf, datapoints, |d, v: f32| {
                            d.temperature = Some(v - 273.15)
                        }),
                        mosmix::WIND_DIR => {
                            parse_into(reader, &mut buf, datapoints, |d, v| d.wind_dir = Some(v))
                        }
                        mosmix::WIND_SPEED => {
                            parse_into(reader, &mut buf, datapoints, |d, v| d.mean_wind = Some(v))
                        }
                        mosmix::WIND_GUSTS => {
                            parse_into(reader, &mut buf, datapoints, |d, v| d.wind_gusts = Some(v))
                        }
                        _ => (),
                    }
                }
            }
            Event::End(e) if e.name().as_ref() == b"kml:ExtendedData" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(())
}

fn parse_into<T: std::str::FromStr>(
    reader: &mut quick_xml::Reader<impl io::BufRead>,
    buf: &mut Vec<u8>,
    datapoints: &mut [Datapoint],
    mut sel: impl FnMut(&mut Datapoint, T),
) {
    let Some(Event::Text(t)) = dig_to_value(reader, buf) else {
        return;
    };
    let Ok(str) = str::from_utf8(&t) else {
        return;
    };
    for (val, dp) in str.split_whitespace().zip(datapoints.iter_mut()) {
        if val != "-" {
            let Ok(v) = val.parse() else {
                continue;
            };
            sel(dp, v);
        }
    }
}

fn parse_sig_weather_into(
    reader: &mut quick_xml::Reader<impl io::BufRead>,
    buf: &mut Vec<u8>,
    datapoints: &mut [Datapoint],
) {
    let Some(Event::Text(t)) = dig_to_value(reader, buf) else {
        return;
    };
    let Ok(str) = str::from_utf8(&t) else {
        return;
    };
    for (mut val, dp) in str.split_whitespace().zip(datapoints.iter_mut()) {
        if let Some(v) = val.strip_suffix(".00") {
            val = v
        }
        let Ok(v) = val.parse() else {
            continue;
        };
        dp.condition = WeatherCondition::Kml(v);
    }
}

fn dig_to_value<'a>(
    reader: &mut quick_xml::Reader<impl io::BufRead>,
    buf: &'a mut Vec<u8>,
) -> Option<Event<'a>> {
    buf.clear();
    let Ok(Event::Start(t)) = reader.read_event_into(buf) else {
        return None;
    };
    if t.name().0 != b"dwd:value" {
        return None;
    }
    buf.clear();
    reader.read_event_into(buf).ok()
}

// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/kml/mosmix_elemente_xls.xlsx?__blob=publicationFile&v=7
mod mosmix {
    pub const SIGNIFICANT_WEATHER: &[u8] = b"ww";
    pub const PRECIPITATION: &[u8] = b"RR1c";
    pub const PRECIPITATION_P: &[u8] = b"wwP"; // only mosmixL
    pub const EFFECTIVE_CLOUD_COVER: &[u8] = b"Neff";
    pub const TEMP_2M: &[u8] = b"TTT";
    pub const WIND_DIR: &[u8] = b"DD";
    pub const WIND_SPEED: &[u8] = b"FF";
    pub const WIND_GUSTS: &[u8] = b"FX1";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MosmixType {
    L,
    S,
}

impl MosmixType {
    pub fn url(self, station: PoiStation) -> String {
        match self {
            MosmixType::L => format!("https://opendata.dwd.de/weather/local_forecasts/mos/MOSMIX_L/single_stations/{station}/kml/MOSMIX_L_LATEST_{station}.kmz"),
            MosmixType::S => "https://opendata.dwd.de/weather/local_forecasts/mos/MOSMIX_S/all_stations/kml/MOSMIX_S_LATEST_240.kmz".to_owned(),
        }
    }

    pub fn etag(self, cache: &Cache) -> &Option<String> {
        match self {
            MosmixType::L => &cache.forecast_l_etag,
            MosmixType::S => &cache.forecast_s_etag,
        }
    }
}
