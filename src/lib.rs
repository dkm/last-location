pub mod models;
pub mod schema;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
}

use models::{NewInfo, UserLocationPoint, UserInfo};
use diesel::prelude::*;

pub fn get_user(db: &mut SqliteConnection, user_id: i32) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user =
        users.filter(id.eq(user_id))
                .limit(1)
                .select(UserInfo::as_select())
                .load(db);

    match user {
        Ok(mut ui) => ui.pop(),
        Err(_) => None,
    }
}

pub fn add_info(db: &mut SqliteConnection, new_info: NewInfo) -> Result<UserLocationPoint, ApiError>{
    use schema::info;

    get_user(db, new_info.user_id).ok_or(ApiError::NotFound)?;

    let res =
            diesel::insert_into(info::table)
                .values(&new_info)
                .returning(UserLocationPoint::as_returning())
                .get_result(db);

    match res {
        Ok(ulp) => Ok(ulp),
        Err(_) => Err(ApiError::NotFound),
    }
}

pub fn get_last_info(db: &mut SqliteConnection, uid: i32) -> Option<UserLocationPoint> {
    use schema::info::dsl::*;

    let last_pos =
            info.filter(user_id.eq(uid))
                .limit(1)
                .select(UserLocationPoint::as_select())
                .order(id.desc())
                .load(db);

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
