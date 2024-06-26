pub mod models;
pub mod schema;

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Error {
    NotFound,
    UserNotFound,
    Undefined,
}

use diesel::{prelude::*, sql_query};
use models::{NewInfo, NewInfoSec, UserInfo, UserLocationPoint};

use crate::models::{NewUser, UserLocationPointSec};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn init(db: &mut SqliteConnection) -> Result<(), Error> {
    match sql_query("PRAGMA foreign_keys = ON").execute(db) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Undefined),
    }
}

pub fn run_migrations(db: &mut SqliteConnection) -> Result<(), Error> {
    // FIXME
    match db.run_pending_migrations(MIGRATIONS) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Undefined),
    }
}

pub fn set_unique_url(
    db: &mut SqliteConnection,
    user_id: i32,
    uniq_url: &str,
) -> Result<(), Error> {
    use schema::users::dsl::*;

    let r = diesel::update(users)
        .filter(id.eq(user_id))
        .set(unique_url.eq(uniq_url))
        .execute(db);
    match r {
        Ok(1) => Ok(()),
        Ok(0) => Err(Error::UserNotFound),
        Ok(_) | Err(_) => Err(Error::Undefined),
    }
}

pub fn get_all_users(db: &mut SqliteConnection) -> Option<Vec<UserInfo>> {
    use schema::users::dsl::*;

    let user = users.select(UserInfo::as_select()).load(db);
    match user {
        Ok(ui) => Some(ui),
        Err(_) => None,
    }
}

pub fn get_user_from_url(db: &mut SqliteConnection, uniq_url: &str) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user = users
        .filter(unique_url.eq(uniq_url))
        .select(UserInfo::as_select())
        .load(db);

    match user {
        Ok(mut ui) => {
            if ui.len() == 1 {
                ui.pop()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn get_user_from_id(db: &mut SqliteConnection, uid: i32) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user = users
        .filter(id.eq(uid))
        .select(UserInfo::as_select())
        .load(db);

    match user {
        Ok(mut ui) => {
            if ui.len() == 1 {
                ui.pop()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn get_user_from_token(db: &mut SqliteConnection, token: &str) -> Option<UserInfo> {
    use schema::users::dsl::*;

    let user = users
        .filter(priv_token.eq(token))
        .select(UserInfo::as_select())
        .load(db);

    match user {
        Ok(mut ui) => {
            if ui.len() == 1 {
                ui.pop()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn generate_user_token(db: &mut SqliteConnection, user_id: i32) -> Result<String, Error> {
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
        Ok(1) => Ok(s),
        Ok(0) => Err(Error::UserNotFound),
        Ok(_) | Err(_) => Err(Error::Undefined),
    }
}

pub fn create_user(db: &mut SqliteConnection, name: &str) -> Result<UserInfo, Error> {
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
        Err(_) => Err(Error::Undefined),
    }
}

pub fn delete_user(db: &mut SqliteConnection, user_id: i32) -> Result<(), Error> {
    use schema::users;
    use schema::users::dsl::*;

    let res = diesel::delete(users::table)
        .filter(id.eq(user_id))
        .execute(db);

    match res {
        Ok(1) => Ok(()),
        Ok(0) => Err(Error::UserNotFound),
        Ok(_) | Err(_) => Err(Error::Undefined),
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
        Ok(mut ui) => {
            if ui.len() == 1 {
                ui.pop()
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn add_info(db: &mut SqliteConnection, new_info: NewInfo) -> Result<UserLocationPoint, Error> {
    use schema::info;

    get_user(db, new_info.user_id).ok_or(Error::NotFound)?;

    let res = diesel::insert_into(info::table)
        .values(&new_info)
        .returning(UserLocationPoint::as_returning())
        .get_result(db);

    match res {
        Ok(ulp) => Ok(ulp),
        Err(_) => Err(Error::Undefined),
    }
}

pub fn add_info_sec(db: &mut SqliteConnection, new_info: NewInfoSec) -> Result<(), Error> {
    use schema::info_sec;

    get_user(db, new_info.user_id).ok_or(Error::NotFound)?;

    let res = diesel::insert_into(info_sec::table)
        .values(&new_info)
        .execute(db);

    match res {
        Ok(ulp) => Ok(()),
        Err(_) => Err(Error::Undefined),
    }
}

fn filter_last_info(values: Vec<UserLocationPoint>, time_gap_secs: i32) -> Vec<UserLocationPoint> {
    let mut prev = 0;
    let mut cut = false;

    values
        .into_iter()
        .rev()
        .filter(|x| {
            if cut {
                false
            } else if prev == 0 {
                prev = x.device_timestamp;
                true
            } else {
                let pprev = prev;
                prev = x.device_timestamp;
                if pprev - prev > time_gap_secs {
                    cut = true;
                    false
                } else {
                    true
                }
            }
        })
        .rev()
        .collect()
}

fn filter_last_info_sec(
    values: Vec<UserLocationPointSec>,
    time_gap_secs: i32,
) -> Vec<UserLocationPointSec> {
    let mut prev = 0;
    let mut cut = false;

    values
        .into_iter()
        .rev()
        .filter(|x| {
            if cut {
                false
            } else if prev == 0 {
                prev = x.server_timestamp;
                true
            } else {
                let pprev = prev;
                prev = x.server_timestamp;
                if pprev - prev > time_gap_secs {
                    cut = true;
                    false
                } else {
                    true
                }
            }
        })
        .rev()
        .collect()
}

pub fn get_last_info(
    db: &mut SqliteConnection,
    uid: i32,
    count: i64,
    cut_last_segment: bool,
) -> Option<Vec<UserLocationPoint>> {
    use schema::info::dsl::*;

    let last_pos = info
        .filter(user_id.eq(uid))
        .limit(count)
        .select(UserLocationPoint::as_select())
        .order(id.desc())
        .load(db);

    match last_pos {
        Ok(found_pos) => {
            if !found_pos.is_empty() {
                if cut_last_segment {
                    Some(filter_last_info(found_pos, 12 * 3600))
                } else {
                    Some(found_pos)
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

pub fn get_last_info_sec(
    db: &mut SqliteConnection,
    uid: i32,
    count: i64,
    cut_last_segment: bool,
) -> Option<Vec<UserLocationPointSec>> {
    use schema::info_sec::dsl::*;

    let last_pos = info_sec
        .filter(user_id.eq(uid))
        .limit(count)
        .select(UserLocationPointSec::as_select())
        .order(id.desc())
        .load(db);

    match last_pos {
        Ok(found_pos) => {
            if !found_pos.is_empty() {
                if cut_last_segment {
                    Some(filter_last_info_sec(found_pos, 12 * 3600))
                } else {
                    Some(found_pos)
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
