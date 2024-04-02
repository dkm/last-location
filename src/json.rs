use crate::models::UserLocationPoint;

use rocket::serde::json::{json, Json, Value};
use rocket::{catch, catchers, get, routes};

use crate::Db;

#[get("/<user_id>", format = "json")]
async fn index(db: Db, user_id: i32) -> Option<Json<UserLocationPoint>> {
    match crate::get_last_info(&db, user_id).await {
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
    })
}
