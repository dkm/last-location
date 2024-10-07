use last_position::models::{LogInfo, LogLocationPoint, LogLocationPointSec};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct APILogInfo {
    pub priv_token: Option<String>,
    pub unique_url: Option<String>,
    pub last_activity: Option<i32>,
}

impl From<LogInfo> for APILogInfo {
    fn from(item: LogInfo) -> Self {
        APILogInfo {
            priv_token: item.priv_token,
            unique_url: item.unique_url,
            last_activity: item.last_activity,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct APILogLocationPoint {
    pub device_timestamp: i32,
    pub server_timestamp: i32,

    pub lat: f64,
    pub lon: f64,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

impl From<LogLocationPoint> for APILogLocationPoint {
    fn from(item: LogLocationPoint) -> Self {
        APILogLocationPoint {
            device_timestamp: item.device_timestamp,
            server_timestamp: item.server_timestamp,

            lat: item.lat,
            lon: item.lon,
            altitude: item.altitude,
            speed: item.speed,
            direction: item.direction,

            accuracy: item.accuracy,

            loc_provider: item.loc_provider,
            battery: item.battery,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct APILogLocationPointSec {
    // See you in 2038...
    pub server_timestamp: i32,
    pub data: Vec<u8>,
}

impl From<LogLocationPointSec> for APILogLocationPointSec {
    fn from(item: LogLocationPointSec) -> Self {
        APILogLocationPointSec {
            server_timestamp: item.server_timestamp,
            data: item.data,
        }
    }
}
