use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

use anyhow::anyhow;
use icons::IconSet;
use itertools::Itertools;

use crate::option_ext::OptionExt;

pub mod forecast;
pub mod icons;
mod option_ext;
pub mod radar;
pub mod report;
pub mod synoptic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoiStation(pub u16);

impl std::fmt::Display for PoiStation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(bincode::Encode, bincode::Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherCondition {
    /// present weather
    /// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/csv/poi_present_weather_zuordnung_pdf.pdf
    Poi(u16),
    /// significant weather
    /// https://www.dwd.de/DE/leistungen/opendata/help/schluessel_datenformate/kml/mosmix_element_weather_xls.xlsx
    Kml(u16),
    /// significant weather 0 20 003
    /// https://www.dwd.de/DE/leistungen/pbfb_verlag_vub/pdf_einzelbaende/vub_2_binaer_barrierefrei.pdf
    Synop(u16),
    None,
}

impl WeatherCondition {
    pub fn is_none(&self) -> bool {
        *self == WeatherCondition::None
    }
}

#[derive(bincode::Encode, bincode::Decode, Debug, Clone, PartialEq)]
pub struct Datapoint {
    #[bincode(with_serde)]
    pub timestamp: jiff::Timestamp,
    #[bincode(with_serde)]
    pub local_ts: jiff::Zoned,
    pub condition: WeatherCondition,
    /// In °C
    pub temperature: Option<f32>,
    /// In mm
    pub precipitation: Option<f32>,
    /// In %
    pub p_precipitation: Option<f32>,
    /// In %
    pub cloud_cover: Option<f32>,
    /// In %
    pub relative_humidity: Option<f32>,
    /// In Km/h
    pub mean_wind: Option<f32>,
    /// In Km/h
    pub wind_gusts: Option<f32>,
    /// In °
    pub wind_dir: Option<f32>,

    pub is_report: bool,
}

#[derive(bincode::Encode, bincode::Decode, Debug, Clone, PartialEq)]
pub struct RadarReading {
    #[bincode(with_serde)]
    pub timestamp: jiff::Timestamp,
    #[bincode(with_serde)]
    pub local_ts: jiff::Zoned,
    // In mm/h
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub poi_station: PoiStation,
    pub radar_coords: (usize, usize),
    pub synop_stations: Vec<String>,
}

pub static ZONE: LazyLock<jiff::tz::TimeZone> = LazyLock::new(jiff::tz::TimeZone::system);

impl Datapoint {
    pub fn from_timestamp(timestamp: jiff::Timestamp, is_report: bool) -> Self {
        Self {
            timestamp,
            local_ts: timestamp.to_zoned(ZONE.clone()),
            condition: WeatherCondition::None,
            temperature: None,
            precipitation: None,
            p_precipitation: None,
            cloud_cover: None,
            relative_humidity: None,
            mean_wind: None,
            wind_gusts: None,
            wind_dir: None,
            is_report,
        }
    }

    pub fn merge_from(&mut self, other: &Datapoint) {
        if self.condition.is_none() {
            self.condition = other.condition;
        }
        self.temperature.or_assign(other.temperature);
        self.precipitation.or_assign(other.precipitation);
        self.p_precipitation.or_assign(other.p_precipitation);
        self.cloud_cover.or_assign(other.cloud_cover);
        self.relative_humidity.or_assign(other.relative_humidity);
        self.mean_wind.or_assign(other.mean_wind);
        self.wind_gusts.or_assign(other.wind_gusts);
        self.wind_dir.or_assign(other.wind_dir);
    }

    pub fn merge_series_ref(left: &[Datapoint], right: &[Datapoint]) -> Vec<Datapoint> {
        left.iter()
            .merge_join_by(right, |a, b| a.timestamp.cmp(&b.timestamp))
            .map(|d| match d {
                itertools::EitherOrBoth::Both(s, l) => {
                    let mut merged = s.clone();
                    merged.merge_from(l);
                    merged
                }
                itertools::EitherOrBoth::Left(it) => it.clone(),
                itertools::EitherOrBoth::Right(it) => it.clone(),
            })
            .collect()
    }

    pub fn merge_series_vec(left: Vec<Datapoint>, right: Vec<Datapoint>) -> Vec<Datapoint> {
        left.into_iter()
            .merge_join_by(right, |a, b| a.timestamp.cmp(&b.timestamp))
            .map(|d| match d {
                itertools::EitherOrBoth::Both(mut s, l) => {
                    s.merge_from(&l);
                    s
                }
                itertools::EitherOrBoth::Left(it) => it,
                itertools::EitherOrBoth::Right(it) => it,
            })
            .collect()
    }

    pub fn icon<I: IconSet>(&self) -> Option<PathBuf> {
        let is_night = self.local_ts.hour() < 7 || self.local_ts.hour() > 20;
        match self.condition {
            WeatherCondition::Poi(pw) => Some(I::present_weather_to_path(pw, is_night)),
            WeatherCondition::Kml(sw) | WeatherCondition::Synop(sw) => Some(
                I::significant_weather_to_path(sw, self.cloud_cover.unwrap_or_default(), is_night),
            ),
            WeatherCondition::None => None,
        }
    }
}

#[derive(bincode::Encode, bincode::Decode, Debug, Default)]
pub struct Cache {
    pub(self) report_etag: Option<String>,
    pub(self) forecast_s_etag: Option<String>,
    pub(self) forecast_l_etag: Option<String>,
    pub(self) radar_etag: Option<String>,
    pub(self) synop_etag: Option<String>,

    pub report: Vec<Datapoint>,
    pub forecast: Vec<Datapoint>,
    pub radar: Vec<RadarReading>,
    pub observation: Option<Datapoint>,
}

impl Cache {
    pub fn from_file(name: &str) -> anyhow::Result<Self> {
        bincode::decode_from_slice(&std::fs::read(name)?, bincode::config::standard())
            .map_err(Into::into)
            .map(|it| it.0)
    }

    pub fn to_file(&self, name: &str) -> anyhow::Result<()> {
        std::fs::write(
            name,
            &bincode::encode_to_vec(self, bincode::config::standard())?,
        )?;
        Ok(())
    }

    pub fn from_file_or_default(name: &str) -> Self {
        Self::from_file(name).unwrap_or_default()
    }

    pub fn refetch(c: &Arc<RwLock<Self>>, config: &Config) -> anyhow::Result<bool> {
        std::thread::scope(|s| {
            let report_t = s.spawn({
                let cache = c.clone();
                let station = config.poi_station;
                move || report::get(station, &cache)
            });
            let radar_t = s.spawn({
                let cache = c.clone();
                let coords = config.radar_coords;
                move || radar::get(&cache, coords)
            });
            let synop_t = s.spawn({
                let cache = c.clone();
                let stations = &config.synop_stations;
                move || synoptic::get(&cache, stations)
            });
            let forecast = forecast::get(config.poi_station, c)
                .inspect_err(|e| eprintln!("Failed to fetch forecast: {e}"))
                .unwrap_or(false);
            let report = report_t
                .join()
                .map_err(|_| anyhow!("Failed to join"))?
                .inspect_err(|e| eprintln!("Failed to fetch report: {e}"))
                .unwrap_or(false);
            let radar = radar_t
                .join()
                .map_err(|_| anyhow!("Failed to join"))?
                .inspect_err(|e| eprintln!("Failed to fetch radar: {e}"))
                .unwrap_or(false);
            let synop = synop_t
                .join()
                .map_err(|_| anyhow!("Failed to join"))?
                .inspect_err(|e| eprintln!("Failed to fetch synop: {e}"))
                .unwrap_or(false);
            Ok(forecast || report || radar || synop)
        })
    }
}

fn get_etag(res: &ureq::http::Response<ureq::Body>) -> Option<String> {
    res.headers()
        .get("ETag")
        .and_then(|t| t.to_str().ok())
        .map(|s| s.to_owned())
}

fn needs_fetch<U>(url: U, prev_etag: Option<&str>) -> bool
where
    ureq::http::Uri: TryFrom<U>,
    <ureq::http::Uri as TryFrom<U>>::Error: Into<ureq::http::Error>,
{
    if prev_etag.is_none() {
        return true;
    }

    let Ok(res) = ureq::head(url).call() else {
        return true;
    };
    if !res.status().is_success() {
        return true;
    }
    let etag = get_etag(&res);

    if let (Some(etag), Some(prev_etag)) = (&etag, prev_etag) {
        etag != prev_etag
    } else {
        true
    }
}
