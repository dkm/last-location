use rocket::{
    http::{ContentType, Status},
    serde::json,
    Request,
};
use rocket::serde::json::{json};
use rocket::response::{self, Response, Responder};
use std::io::Cursor;

pub enum ApiError {
    NotFound,
}


#[rocket::async_trait]
impl<'r> response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {

        let body = json!({
            "status": "error",
            "reason": "not found",
        });

        let body_str = body.to_string();

        Response::build()
            .sized_body(body_str.len(), Cursor::new(body_str))
            .status(Status{code:404})
            .header(ContentType::JSON)
            .ok()
    }
}
