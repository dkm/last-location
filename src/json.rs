use crate::models::UserLocationPoint;
use crate::responders::ApiError;

use rocket::serde::json::Json;
use rocket::{ get, routes};

use crate::Db;

#[get("/<user_id>", format = "json")]
async fn index(db: Db, user_id: i32) -> Result<Json<UserLocationPoint>, ApiError> {
    match crate::get_last_info(&db, user_id).await {
        Some(pos) => Ok(Json(pos)),
        _ => Err(ApiError::NotFound),
    }
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount("/api/json", routes![index])
//            .register("/api/json", catchers![not_found])
    })
}
