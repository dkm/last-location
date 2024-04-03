use rocket::{
    http::{ContentType, Status},
    serde::json,
    Request,
};
use rocket::serde::json::{json};
use rocket::response::{self, Response, Responder};

pub enum ApiError {
    NotFound,
}


#[rocket::async_trait]
impl<'r> response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let string = "Coucou".to_string();
        Response::build_from(string.respond_to(req)?)
            .status(Status{code:404})
            .header(ContentType::JSON)
            .ok()

//           let (status, body) = match self {
//               ApiError::NotFound => (
//                 Status::NotFound,
//                   json!({
//                       "status": "error",
//                       "reason": "Resource was not found."
//                   }))
//           };

        // response::Response::build_from(body.to_string())
        //     .header(ContentType::JSON)
        //     .status(status)

        //     .ok()
    }
}
