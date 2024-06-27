use crate::schema::{
    info::{self, accuracy},
    info_sec, users,
};
use hex;
use std::fmt;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserInfo {
    pub id: i32,
    pub name: Option<String>,
    pub priv_token: Option<String>,
    pub unique_url: Option<String>,
}

impl fmt::Display for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "id: {}, name: {}, priv_token: {}, unique_url: {}",
            self.id,
            // FIXME this is ugly.
            self.name.clone().map_or("None".to_string(), |v| v.clone()),
            self.priv_token
                .clone()
                .map_or("None".to_string(), |v| v.clone()),
            self.unique_url
                .clone()
                .map_or("None".to_string(), |v| v.clone()),
        )
    }
}

#[derive(Insertable, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: Option<String>,
}

#[derive(Queryable, Selectable, Clone, serde::Serialize, serde::Deserialize, Debug)]
#[diesel(table_name = info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserLocationPoint {
    pub user_id: i32,
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

impl fmt::Display for UserLocationPoint {
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
            format!("{}", a)
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
pub struct UserLocationPointSec {
    pub user_id: i32,
    pub id: i32,

    // See you in 2038...
    pub server_timestamp: i32,
    pub data: Vec<u8>,
}

impl fmt::Display for UserLocationPointSec {
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
    pub user_id: i32,

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

#[derive(Insertable, serde::Deserialize, Clone, Debug)]
#[diesel(table_name = info_sec)]
pub struct NewInfoSec {
    pub user_id: i32,

    pub server_timestamp: Option<i32>,
    pub data: Vec<u8>,
}
