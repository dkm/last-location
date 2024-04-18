pub mod models;
pub mod schema;

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ApiError {
    NotFound,
}

use diesel::prelude::*;
use models::{NewInfo, UserInfo, UserLocationPoint};

use crate::models::NewUser;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn run_migrations(db: &mut SqliteConnection) -> Result<(), ()> {
    // FIXME
    match db.run_pending_migrations(MIGRATIONS) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn set_unique_url(db: &mut SqliteConnection, user_id: i32, uniq_url: &str) -> Result<(), ()> {
    use schema::users::dsl::*;

    let r = diesel::update(users)
        .filter(id.eq(user_id))
        .set(unique_url.eq(uniq_url))
        .execute(db);
    match r {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

pub fn get_user_from_url(db: &mut SqliteConnection, uniq_url: &str) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user = users
        .filter(unique_url.eq(uniq_url))
        .select(UserInfo::as_select())
        .load(db);

    match user {
        Ok(mut ui) => ui.pop(),
        Err(_) => None,
    }
}

pub fn generate_user_token(db: &mut SqliteConnection, user_id: i32) -> Result<String, ()> {
    use schema::users::dsl::*;

    let s: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let r = diesel::update(users)
        .filter(id.eq(user_id))
        .set(priv_token.eq(&s))
        .execute(db);
    match r {
        Ok(_) => Ok(s),
        Err(_) => Err(()),
    }
}

pub fn create_user(db: &mut SqliteConnection, name: &str) -> Result<UserInfo, ApiError> {
    use schema::users;
    let new_user = NewUser {
        name: Some(String::from(name)),
    };

    let res = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(UserInfo::as_returning())
        .get_result(db);

    match res {
        Ok(nu) => Ok(nu),
        Err(_) => Err(ApiError::NotFound),
    }
}

pub fn get_user(db: &mut SqliteConnection, user_id: i32) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user = users
        .filter(id.eq(user_id))
        .limit(1)
        .select(UserInfo::as_select())
        .load(db);

    match user {
        Ok(mut ui) => ui.pop(),
        Err(_) => None,
    }
}

pub fn add_info(
    db: &mut SqliteConnection,
    new_info: NewInfo,
) -> Result<UserLocationPoint, ApiError> {
    use schema::info;

    get_user(db, new_info.user_id).ok_or(ApiError::NotFound)?;

    let res = diesel::insert_into(info::table)
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

    let last_pos = info
        .filter(user_id.eq(uid))
        .limit(1)
        .select(UserLocationPoint::as_select())
        .order(id.desc())
        .load(db);

    match last_pos {
        Ok(found_pos) => {
            if !found_pos.is_empty() {
                Some(found_pos[0].clone())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
