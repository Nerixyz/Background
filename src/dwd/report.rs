use anyhow::{anyhow, bail};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    sync::RwLock,
};

use crate::dwd::{get_etag, needs_fetch};

use super::{Cache, Datapoint, PoiStation, WeatherCondition};

pub fn get(station: PoiStation, cache: &RwLock<Cache>) -> anyhow::Result<bool> {
    let url = format!("https://opendata.dwd.de/weather/weather_reports/poi/{station}-BEOB.csv");
    if !needs_fetch(&url, cache.read().unwrap().report_etag.as_deref()) {
        return Ok(false);
    }

    let mut res = ureq::get(url).call()?;
    if !res.status().is_success() {
        bail!("Failed to get station - got status {:?}", res.status());
    }
    let etag = get_etag(&res);

    let reader = BufReader::new(res.body_mut().as_reader());
    let mut lines = reader.lines();
    let header = lines
        .next()
        .and_then(Result::ok)
        .ok_or_else(|| anyhow!("No header"))?;
    let columns = HashMap::from_iter(header.split(';').enumerate().map(|(k, v)| (v, k)));
    if lines.next().is_none() || lines.next().is_none() {
        bail!("Too few lines");
    }

    let mut data = lines
        .filter_map(Result::ok)
        .map(|s| s.replace(',', "."))
        .filter_map(|it| {
            Datapoint::from_report(&columns, &it.split(';').collect::<Vec<&str>>()).ok()
        })
        .collect::<Vec<Datapoint>>();

    data.sort_unstable_by_key(|it| it.timestamp);
    let mut cache = cache.write().unwrap();
    cache.report_etag = etag;
    cache.report = data;

    Ok(true)
}

impl Datapoint {
    fn from_report(columns: &HashMap<&str, usize>, fields: &[&str]) -> anyhow::Result<Self> {
        let get = |name: &'static str| -> Option<&str> {
            columns
                .get(name)
                .and_then(|idx| fields.get(*idx).map(|it| &**it))
        };
        let get_f32 =
            |name: &'static str| -> Option<f32> { get(name).and_then(|v| v.parse().ok()) };
        let get_u16 =
            |name: &'static str| -> Option<u16> { get(name).and_then(|v| v.parse().ok()) };
        // parse the timestamp
        let date = get("surface observations").ok_or_else(|| anyhow!("No date column"))?;
        let time = get("Parameter description").ok_or_else(|| anyhow!("No time column"))?;
        let date = jiff::civil::Date::strptime(b"%d.%m.%y", date)?;
        let time = jiff::civil::Time::strptime("%R", time)?;
        let timestamp = date
            .to_datetime(time)
            .to_zoned(jiff::tz::TimeZone::UTC)?
            .timestamp();

        let condition = match get_u16("present_weather") {
            Some(x) => WeatherCondition::Poi(x),
            None => WeatherCondition::None,
        };
        let temperature = get_f32("dry_bulb_temperature_at_2_meter_above_ground");
        let precipitation = get_f32("precipitation_amount_last_hour");
        let cloud_cover = get_f32("cloud_cover_total");
        let relative_humidity = get_f32("relative_humidity");
        let mean_wind = get_f32("mean_wind_speed_during last_10_min_at_10_meters_above_ground");
        let wind_gusts = get_f32("maximum_wind_speed_last_hour");
        let wind_dir = get_f32("mean_wind_direction_during_last_10 min_at_10_meters_above_ground");

        Ok(Self {
            timestamp,
            local_ts: timestamp.to_zoned(super::ZONE.clone()),
            condition,
            temperature,
            precipitation,
            p_precipitation: None,
            cloud_cover,
            relative_humidity,
            mean_wind,
            wind_gusts,
            wind_dir,
            is_report: true,
        })
    }
}
