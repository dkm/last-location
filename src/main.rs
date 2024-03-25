#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::State;

use last_position::{LastInfoPointer, LastPilotInfo};

use dotenvy::dotenv;
use last_position::models::NewInfo;

use diesel::prelude::*;
use std::env;

#[post("/info", data = "<newinfo>")]
fn info(newinfo: Form<NewInfo>, db: &State<LastInfoPointer>) {
    let pinfo: NewInfo = *newinfo;
    let conn = &mut db.lock().unwrap().db;

    last_position::add_info(&pinfo, conn);
}

#[get("/")]
fn index(db: &State<LastInfoPointer>) -> Option<String> {
    let conn = &mut db.lock().unwrap().db;

    last_position::get_last_info(conn).map(|pos| {
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
        .mount("/", routes![index, info])
}
