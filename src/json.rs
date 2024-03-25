use crate::models::PilotInfo;

use rocket::serde::json::{json, Json, Value};
use rocket::{catch, catchers, get, routes, State};

use crate::LastInfoPointer;

#[get("/<pilot_id>", format = "json")]
fn index(db: &State<LastInfoPointer>, pilot_id: i32) -> Option<Json<PilotInfo>> {
    let conn = &mut db.lock().unwrap().db;

    match crate::get_last_info(conn, pilot_id) {
        Some(pos) => Some(Json(pos)),
        _ => None,
    }
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount("/api/json", routes![index])
            .register("/api/json", catchers![not_found])
        //              .manage(LastPilotInfo::new(db))
    })
}
