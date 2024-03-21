#[macro_use] extern crate rocket;
use std::sync::Mutex;
use std::sync::Arc;
use rocket::form::Form;
use rocket::State;

use time::PrimitiveDateTime;
use time::macros::datetime;

#[derive(FromForm, Copy, Clone)]
struct Location {
    lat : f64,
    lon : f64,
    date: PrimitiveDateTime,

}

struct LastLocation {
    pos: Option<Location>,
}

type LastLocPointer = Arc<Mutex<LastLocation>>;

impl LastLocation {
    fn new() -> LastLocPointer {
        let new_loc = LastLocation {
            pos: None
        };
        Arc::new(Mutex::new(new_loc))
    }
}

#[post("/location", data="<location>")]
fn location(location: Form<Location>, last_location: &State<LastLocPointer>) {
    *last_location.lock().unwrap() = LastLocation {
        pos : Some (Location {
            lat: location.lat,
            lon: location.lon,
            date: location.date,
        }),
    }
}

#[get("/")]
fn index(last_location: &State<LastLocPointer>) -> String {
    let loc = last_location.lock().unwrap();
    match loc.pos {
        Some(loc) => format!("lat: {}, lon: {}, date: {}", loc.lat, loc.lon, loc.date),
        None => "No Location".to_string(),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(LastLocation::new())
                   .mount("/", routes![index, location])
}
