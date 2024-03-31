#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::{
    fs::{relative, FileServer},
    State,
};

use last_position::{LastInfoPointer, LastPilotInfo};
use std::time::{SystemTime, UNIX_EPOCH};
use dotenvy::dotenv;
use last_position::models::NewInfo;

use diesel::prelude::*;
use std::env;

#[post("/info", data = "<newinfo>")]
fn info(newinfo: Form<NewInfo>, db: &State<LastInfoPointer>) {
    let mut pinfo: NewInfo = *newinfo;
    // this will get messy in 2038.
    pinfo.server_timestamp = Some(SystemTime::now().duration_since(UNIX_EPOCH).expect("Can't get epoch").as_secs() as i32);
    let conn = &mut db.lock().unwrap().db;
    last_position::add_info(&pinfo, conn);
}

#[get("/<pilot_id>")]
fn index(db: &State<LastInfoPointer>, pilot_id: i32) -> Option<String> {
    let conn = &mut db.lock().unwrap().db;

    last_position::get_last_info(conn, pilot_id).map(|pos| {
        format!(
            "lat:{}, lon:{}, accuracy:{}",
            pos.lat, pos.lon, pos.accuracy
        )
    })
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    rocket::build()
        .manage(LastPilotInfo::new(db))
        .attach(last_position::json::stage())
        .mount("/", FileServer::from(relative!("/static")))
        .mount("/api/", routes![index, info])
}
