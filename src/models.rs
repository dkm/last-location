use crate::schema::{info, users};
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
        write!(f, "(lat:{}, lon:{})", self.lat, self.lon)
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
