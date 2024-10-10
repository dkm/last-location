use crate::schema::{info, info_sec, logs};
use hex;
use std::fmt;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = logs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LogInfo {
    pub id: i32,

    pub priv_token: Option<String>,
    pub unique_url: Option<String>,
    pub last_activity: Option<i32>,
}

impl fmt::Display for LogInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}, priv_token: {}, unique_url: {}, last-activity: {}",
            self.id,
            // FIXME this is ugly.
            //            self.name.clone().map_or("None".to_string(), |v| v.clone()),
            self.priv_token
                .clone()
                .map_or("None".to_string(), |v| v.clone()),
            self.unique_url
                .clone()
                .map_or("None".to_string(), |v| v.clone()),
            self.last_activity
                .map_or("None".to_string(), |v| format!("{}", v.clone())),
        )
    }
}

// #[derive(Insertable, serde::Deserialize, Clone, Debug)]
// #[diesel(table_name = logs)]
// pub struct NewUser {
//     pub name: Option<String>,
// }

#[derive(Queryable, Selectable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LogLocationPoint {
    pub log_id: i32,
    pub id: i32,

    // See you in 2038...
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

impl fmt::Display for LogLocationPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alt = if let Some(a) = self.altitude {
            format!("{}", a)
        } else {
            "None".to_string()
        };
        let speed = if let Some(a) = self.speed {
            format!("{}", a)
        } else {
            "None".to_string()
        };
        let direction = if let Some(a) = self.direction {
            format!("{}", a)
        } else {
            "None".to_string()
        };
        let other_accuracy = if let Some(a) = self.accuracy {
            format!("{}", a)
        } else {
            "None".to_string()
        };
        let prov = if let Some(a) = self.loc_provider.as_ref() {
            a.to_string()
        } else {
            "None".to_string()
        };
        let bat = if let Some(a) = self.battery {
            format!("{}", a)
        } else {
            "None".to_string()
        };

        write!(f, "(dev ts:{}, srv ts:{}, lat:{}, lon:{}, alt:{}, speed:{}, dir:{}, acc:{}, prov:{}, bat:{} )",
               self.device_timestamp,
               self.server_timestamp,
               self.lat,
               self.lon,
               alt,
               speed, direction, other_accuracy, prov, bat)
    }
}

#[derive(Queryable, Selectable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = info_sec)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LogLocationPointSec {
    pub log_id: i32,
    pub id: i32,

    // See you in 2038...
    pub server_timestamp: i32,
    pub data: Vec<u8>,
}

impl fmt::Display for LogLocationPointSec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(srv ts:{}, data: 0x{})",
            self.server_timestamp,
            hex::encode(&self.data),
        )
    }
}

#[derive(Insertable, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = info)]
pub struct NewInfo {
    pub log_id: i32,

    pub device_timestamp: i32,
    pub server_timestamp: Option<i32>,

    pub lat: f64,
    pub lon: f64,

    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

impl NewInfo {
    pub fn is_valid(&self) -> bool {
        // This make the testing harder for a probable very little added value.
        //
        // if let Some(server_ts) = &self.server_timestamp {
        //     if (self.device_timestamp - server_ts).abs() > 24 * 3600 {
        //         return false;
        //     }
        // }

        if self.lat < -90f64 || self.lat > 90f64 {
            return false;
        }

        if self.lon < -180f64 || self.lon > 180f64 {
            return false;
        }

        if let Some(alt) = &self.altitude {
            if *alt < -500f64 || *alt > 10000f64 {
                return false;
            }
        }

        true
    }
}

#[derive(Insertable, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = info_sec)]
pub struct NewInfoSec {
    pub log_id: i32,

    pub server_timestamp: Option<i32>,

    pub data: Vec<u8>,
}
