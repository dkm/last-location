use diesel::prelude::*;
use time::PrimitiveDateTime;

use crate::schema::info;

#[derive(Queryable, Selectable, FromForm, Copy, Clone)]
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
