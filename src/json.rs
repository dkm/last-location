use crate::models::PilotInfo;

use rocket::serde::json::{json, Json, Value};
use rocket::{State, catchers, routes, catch, get};

use crate::LastInfoPointer;

#[get("/", format = "json")]
fn index(db: &State<LastInfoPointer>) -> Option<Json<PilotInfo>> {
    let conn = &mut db.lock().unwrap().db;

    match crate::get_last_info(conn) {
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
            .mount("/json", routes![index])
            .register("/json", catchers![not_found])
        //              .manage(LastPilotInfo::new(db))
    })
}
