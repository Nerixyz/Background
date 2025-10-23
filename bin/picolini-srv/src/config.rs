use std::path::PathBuf;
use std::sync::LazyLock;

use base64::Engine;
use dwd_fetch::PoiStation;

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigData {
    station: u16,
    latitude: f64,
    longitude: f64,
    synop_stations: Vec<String>,
    secret: String,
    access_secret: String,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    host: Option<String>,
}

pub struct Config {
    dwd: dwd_fetch::Config,
    secret: [u8; 512],
    access_secret: String, // to view the past x readings
    port: u16,
    host: String,
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
        let radar_coords = dwd_fetch::latlong_to_idx(data.latitude, data.longitude);
        let dwd = dwd_fetch::Config {
            poi_station: PoiStation(data.station),
            radar_coords,
            synop_stations: data.synop_stations,
        };
        let secret = base64::prelude::BASE64_STANDARD
            .decode(&data.secret)
            .unwrap();
        Self {
            dwd,
            secret: secret.try_into().unwrap(),
            access_secret: data.access_secret,
            port: data.port.unwrap_or(8080),
            host: data.host.unwrap_or_else(|| "127.0.0.1".into()),
        }
    }

    pub fn dwd(&self) -> &dwd_fetch::Config {
        &self.dwd
    }

    pub fn secret(&self) -> &[u8; 512] {
        &self.secret
    }

    pub fn access_secret(&self) -> &str {
        &self.access_secret
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

fn default_config_paths() -> [PathBuf; 1] {
    [PathBuf::from("config.toml")]
}
