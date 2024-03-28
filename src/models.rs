use crate::schema::info;

use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable, FromForm, Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PilotInfo {
    pub pilot_id: i32,
    pub id: i32,

    pub ts: i32,
    pub lat: f64,
    pub lon: f64,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

#[derive(Insertable, FromForm, Copy, Clone)]
#[diesel(table_name = info)]
pub struct NewInfo<'a> {
    pub pilot_id: i32,

    pub ts: Option<i32>,

    pub lat: f64,
    pub lon: f64,

    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<&'a str>,
    pub battery: Option<f64>,
}
