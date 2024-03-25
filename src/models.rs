use crate::schema::info;

use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable, FromForm, Copy, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PilotInfo {
    pub id: i32,

    pub lat: f64,
    pub lon: f64,

    pub accuracy: i32,
    pub ts: PrimitiveDateTime,
}

#[derive(Insertable, FromForm, Copy, Clone)]
#[diesel(table_name = info)]
pub struct NewInfo {
    pub id: i32,
    pub lat: f64,
    pub lon: f64,
    pub accuracy: i32,
    pub ts: PrimitiveDateTime,
}
