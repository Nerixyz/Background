use std::path::PathBuf;
use std::sync::LazyLock;

use dwd_fetch::PoiStation;

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigData {
    station: u16,
    latitude: f64,
    longitude: f64,
    cache_file: String,
    monitor_at_pos: (i32, i32),
    synop_stations: Vec<String>,
}

pub struct Config {
    dwd: dwd_fetch::Config,
    cache_file: String,
    monitor_at_pos: (i32, i32),
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    for path in default_config_paths() {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let data = toml::from_slice::<ConfigData>(&data).expect("Invalid config");
        return Config::new(data);
    }
    panic!("No config found")
});

impl Config {
    fn new(data: ConfigData) -> Self {
        let radar_coords = latlong_to_idx(data.latitude, data.longitude);
        let dwd = dwd_fetch::Config {
            poi_station: PoiStation(data.station),
            radar_coords,
            synop_stations: data.synop_stations,
        };
        Self {
            dwd,
            monitor_at_pos: data.monitor_at_pos,
            cache_file: data.cache_file,
        }
    }

    pub fn cache_file(&self) -> &str {
        &self.cache_file
    }

    pub fn monitor_at_pos(&self) -> (i32, i32) {
        self.monitor_at_pos
    }

    pub fn dwd(&self) -> &dwd_fetch::Config {
        &self.dwd
    }
}

fn latlong_to_idx(lat: f64, long: f64) -> (usize, usize) {
    let proj = proj4rs::Proj::from_user_string("+proj=stere +lat_0=90 +lat_ts=60 +lon_0=10 +a=6378137 +b=6356752.3142451802 +no_defs +x_0=543196.83521776402 +y_0=3622588.8619310018").unwrap();
    let latlon = proj4rs::Proj::from_user_string("+proj=latlong").unwrap();
    let mut p = (long.to_radians(), lat.to_radians());
    proj4rs::transform::transform(&latlon, &proj, &mut p).unwrap();
    let (x, y) = p;
    // for some reason, y is negative
    (
        (x.round() / 1000.0) as usize,
        (-(y.round() / 1000.0)) as usize,
    )
}

#[cfg(windows)]
fn default_config_paths() -> [PathBuf; 2] {
    [PathBuf::from("config.toml"), {
        let mut appdata = PathBuf::from(
            std::env::var_os("APPDATA").unwrap_or_else(|| "~\\AppData\\Roaming".into()),
        );
        appdata.push("MyBackground/config.toml");
        appdata
    }]
}

#[cfg(unix)]
fn default_config_paths() -> [PathBuf; 1] {
    [{
        let mut cfg_home =
            PathBuf::from(std::env::var_os("XDG_CONFIG_HOME").unwrap_or_else(|| {
                let mut cfg_home = std::env::var_os("HOME").unwrap_or_else(|| "~".into());
                cfg_home.push("/.config");
                cfg_home
            }));
        cfg_home.push("MyBackground/config.toml");
        cfg_home
    }]
}
