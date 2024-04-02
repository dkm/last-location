#[macro_use]
extern crate rocket;
use last_position::models::NewInfo;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::{Build, Rocket};

use last_position::Db;
use rocket::fs::{relative, FileServer};
use std::time::{SystemTime, UNIX_EPOCH};
//use dotenvy::dotenv;

use std::env;

#[post("/info", data = "<newinfo>")]
async fn info(db: Db, newinfo: Form<NewInfo>) {
    let mut pinfo: NewInfo = newinfo.clone();
    // this will get messy in 2038.
    pinfo.server_timestamp = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't get epoch")
            .as_secs() as i32,
    );

    last_position::add_info(db, pinfo).await;
}

#[get("/<user_id>")]
async fn index(db: Db, user_id: i32) -> Option<String> {
    last_position::get_last_info(&db, user_id).await.map(|pos| {
        format!(
            "lat:{}, lon:{}, accuracy:{}",
            pos.lat,
            pos.lon,
            (if let Some(acc) = pos.accuracy {
                acc.to_string()
            } else {
                "None".to_string()
            })
        )
    })
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    Db::get_one(&rocket)
        .await
        .expect("database connection")
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS)
                .expect("diesel migrations");
        })
        .await;

    rocket
}

#[launch]
fn rocket() -> _ {
    // dotenv().ok();
    rocket::build()
        .attach(Db::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        // .manage(LastPilotInfo::new(db))
        .attach(last_position::json::stage())
        .mount("/", FileServer::from(relative!("/static")))
        .mount("/api/", routes![index, info])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/api/1")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().unwrap(),
            "lat:45.20283236, lon:5.75199348, accuracy:14.032693862915039"
        );
    }
}
