pub mod json;
pub mod models;
pub mod schema;

use crate::models::{NewInfo, PilotInfo};

use diesel::prelude::*;
use diesel::SqliteConnection;
use std::sync::Arc;
use std::sync::Mutex;

pub struct LastPilotInfo {
    pub db: SqliteConnection,
}

pub type LastInfoPointer = Arc<Mutex<LastPilotInfo>>;

impl LastPilotInfo {
    pub fn new(db: SqliteConnection) -> LastInfoPointer {
        Arc::new(Mutex::new(LastPilotInfo { db }))
    }
}

pub fn add_info(new_info: &NewInfo, conn: &mut SqliteConnection) {
    use schema::info;

    let _: PilotInfo = diesel::insert_into(info::table)
        .values(new_info)
        .returning(PilotInfo::as_returning())
        .get_result(conn)
        .expect("Error saving new info");
}

pub fn get_last_info(conn: &mut SqliteConnection, pilot_id: i32) -> Option<PilotInfo> {
    use schema::info::dsl::*;

    let last_pos = info
        .filter(id.eq(pilot_id))
        .limit(1)
        .select(PilotInfo::as_select())
        .order(ts.desc())
        .load(conn);

    match last_pos {
        Ok(pos) => {
            if pos.len() > 0 {
                Some(pos[0])
            } else {
                None
            }
        }
        _ => None,
    }
}
