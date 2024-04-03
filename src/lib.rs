pub mod json;
pub mod models;
pub mod schema;
pub mod responders;


use models::{NewInfo, UserLocationPoint};
use diesel::prelude::*;
use rocket_sync_db_pools::{database, diesel};
use responders::ApiError;

#[database("sqlite_info")]
pub struct Db(diesel::SqliteConnection);

pub async fn add_info(db: Db, new_info: NewInfo) -> Result<(), ApiError>{
    use schema::info;

    let res = db.run({
        let info = new_info.clone();
        move |conn| {
            diesel::insert_into(info::table)
                .values(&info)
                .returning(UserLocationPoint::as_returning())
                .get_result(conn)
        }
    })
    .await;

    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::NotFound),
    }
}

pub async fn get_last_info(db: &Db, uid: i32) -> Option<UserLocationPoint> {
    use schema::info::dsl::*;

    let last_pos = db
        .run(move |conn| {
            info.filter(user_id.eq(uid))
                .limit(1)
                .select(UserLocationPoint::as_select())
                .order(id.desc())
                .load(conn)
        })
        .await;

    match last_pos {
        Ok(found_pos) => {
            if found_pos.len() > 0 {
                Some(found_pos[0].clone())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
