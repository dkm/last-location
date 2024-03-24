#[macro_use]
extern crate rocket;
use rocket::form::Form;
use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;
use models::{NewInfo, PilotInfo};
use rocket::serde::json::{Json, Value, json};

use std::env;

struct LastPilotInfo {
    db: SqliteConnection,
}

type LastInfoPointer = Arc<Mutex<LastPilotInfo>>;

impl LastPilotInfo {
    fn new(db: SqliteConnection) -> LastInfoPointer {
        Arc::new(Mutex::new(LastPilotInfo { db }))
    }
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[post("/info", data = "<newinfo>")]
fn info(newinfo: Form<NewInfo>, db: &State<LastInfoPointer>) {
    use schema::info;

    let pinfo: NewInfo = *newinfo;
    let conn = &mut db.lock().unwrap().db;

    let _: PilotInfo = diesel::insert_into(info::table)
        .values(&pinfo)
        .returning(PilotInfo::as_returning())
        .get_result(conn)
        .expect("Error saving new info");
}

#[get("/", format = "json")]
fn index(db: &State<LastInfoPointer>) -> Option<Json<PilotInfo>>  {
    use schema::info;
    use schema::info::dsl::*;

    let conn = &mut db.lock().unwrap().db;

    let last_pos = info
        .filter(id.eq(1))
        .limit(1)
        .select(PilotInfo::as_select())
        .order(ts.desc())
        .load(conn);

    match last_pos {
        Ok(pos) => Some(Json(pos[0])),
        _ => None,
    }
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    rocket::build()
        .manage(LastPilotInfo::new(db))
        .mount("/", routes![index, info])
}
