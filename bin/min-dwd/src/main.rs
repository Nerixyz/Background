use std::sync::{Arc, RwLock};

fn main() {
    let data = std::fs::read("config.toml").unwrap();
    let data = toml::from_slice::<ConfigData>(&data).expect("Invalid config");
    let radar_coords = dwd_fetch::latlong_to_idx(data.latitude, data.longitude);
    let dwd = dwd_fetch::Config {
        poi_station: dwd_fetch::PoiStation(data.station),
        radar_coords,
        synop_stations: data.synop_stations,
    };

    let cache = Arc::new(RwLock::new(dwd_fetch::Cache::default()));
    dwd_fetch::Cache::refetch(&cache, &dwd).unwrap();
    std::hint::black_box(&cache.read().unwrap());
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigData {
    station: u16,
    latitude: f64,
    longitude: f64,
    synop_stations: Vec<String>,
}
