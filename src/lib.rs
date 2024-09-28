pub mod models;
pub mod schema;

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::{fs::File, io::BufReader};
use word_generator::*;

#[derive(thiserror::Error, Debug)]
#[error("Generic error message")]
pub enum Error {
    // Diesel error
    DatabaseError(#[from] Box<dyn core::error::Error + Sync + Send>),
    DatabaseError2(#[from] diesel::result::Error),
    NotFound,
    LogNotFound,
    Undefined,
}

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

use diesel::{prelude::*, sql_query};
use models::{LogInfo, LogLocationPoint, NewInfo, NewInfoSec};

use crate::models::LogLocationPointSec;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Initialize the DBMS. For SQLite, it enable the `PRAGMA foreign_keys`.
pub fn init(db: &mut SqliteConnection) -> Result<(), Error> {
    sql_query("PRAGMA foreign_keys = ON").execute(db)?;
    Ok(())
}

/// Run the migrations from the 'migrations/' directory
pub fn run_migrations(db: &mut SqliteConnection) -> Result<(), Error> {
    db.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

/// Sets the unique URL for a given log.
pub fn set_unique_url(db: &mut SqliteConnection, log_id: i32, uniq_url: &str) -> Result<(), Error> {
    use schema::logs::dsl::*;

    let r = diesel::update(logs)
        .filter(id.eq(log_id))
        .set(unique_url.eq(uniq_url))
        .execute(db);
    match r {
        Ok(1) => Ok(()),
        Ok(0) => Err(Error::LogNotFound),
        Ok(_) => Err(Error::Undefined),
        Err(e) => Err(e.into()),
    }
}

/// Generates a random URL and sets it as the unique URL for a given log.
pub fn generate_random_url(db: &mut SqliteConnection, log_id: i32) -> Result<(), Error> {
    // 50 tries. If that doesn't work, maybe it's time to exit...
    for _ in 1..50 {
        let reader = BufReader::new(File::open("list").unwrap()); // using your language
                                                                  // let reader = BufReader::new(langs::FR_TXT);
        let suffix: u8 = rand::random::<u8>() % 100;

        let maybe_url = format!(
            "{}{}",
            generate_words(reader, 3, 1).expect("Error generating word")[0],
            suffix
        );

        if set_unique_url(db, log_id, &maybe_url).is_ok() {
            return Ok(());
        }
    }
    Err(Error::Undefined)
}

/// Gets all the active logs.
pub fn get_all_logs(db: &mut SqliteConnection) -> Option<Vec<LogInfo>> {
    use schema::logs::dsl::*;
    logs.select(LogInfo::as_select()).load(db).ok()
}

/// Gets a log from its URL
pub fn get_log_from_url(db: &mut SqliteConnection, uniq_url: &str) -> Option<LogInfo> {
    use schema::logs::dsl::*;

    logs.filter(unique_url.eq(uniq_url))
        .select(LogInfo::as_select())
        .load(db)
        .map_or(None, |mut l| if l.len() == 1 { l.pop() } else { None })
}

/// Gets a log from its id.
pub fn get_log_from_id(db: &mut SqliteConnection, uid: i32) -> Option<LogInfo> {
    use schema::logs::dsl::*;

    logs.filter(id.eq(uid))
        .select(LogInfo::as_select())
        .load(db)
        .map_or(None, |mut l| if l.len() == 1 { l.pop() } else { None })
}

/// Gets a log from its private token.
pub fn get_log_from_token(db: &mut SqliteConnection, token: &str) -> Option<LogInfo> {
    use schema::logs::dsl::*;

    logs.filter(priv_token.eq(token))
        .select(LogInfo::as_select())
        .load(db)
        .map_or(None, |mut l| if l.len() == 1 { l.pop() } else { None })
}

/// Generates a private token and sets it for the given log.
pub fn generate_log_token(db: &mut SqliteConnection, log_id: i32) -> Result<String, Error> {
    use schema::logs::dsl::*;

    let s: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let r = diesel::update(logs)
        .filter(id.eq(log_id))
        .set(priv_token.eq(&s))
        .execute(db);

    match r {
        Ok(1) => Ok(s),
        Ok(0) => Err(Error::LogNotFound),
        Ok(_) => Err(Error::Undefined),
        Err(e) => Err(e.into()),
    }
}

/// Generates a new log.
/// The resulting log has its unique URL and private token set.
pub fn generate_new_log(
    db: &mut SqliteConnection,
    with_token_and_url: bool,
) -> Result<LogInfo, Error> {
    use schema::logs;

    let res = diesel::insert_into(logs::table)
        .default_values()
        .returning(LogInfo::as_returning())
        .get_result(db);

    let Ok(new_log) = res else {
        return Err(Error::Undefined);
    };

    if with_token_and_url {
        if generate_log_token(db, new_log.id).is_err() {
            return Err(Error::Undefined);
        }

        if generate_random_url(db, new_log.id).is_err() {
            return Err(Error::Undefined);
        }
    }

    Ok(new_log)
}

/// Deletes the given log.
pub fn delete_log(db: &mut SqliteConnection, log_id: i32) -> Result<(), Error> {
    use schema::logs;
    use schema::logs::dsl::*;

    let log = diesel::delete(logs::table)
        .filter(id.eq(log_id))
        .execute(db);

    match log {
        Ok(1) => Ok(()),
        Ok(0) => Err(Error::LogNotFound),
        Ok(_) | Err(_) => Err(Error::Undefined),
    }
}

/// Gets the given log.
pub fn get_log(db: &mut SqliteConnection, log_id: i32) -> Option<LogInfo> {
    use schema::logs::dsl::*;

    let log = logs
        .filter(id.eq(log_id))
        .limit(1)
        .select(LogInfo::as_select())
        .load(db);

    match log {
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

/// Sets the last activity timestamp for a given log.
pub fn set_last_activity(
    db: &mut SqliteConnection,
    log_id: i32,
    timestamp: i32,
) -> Result<(), Error> {
    use schema::logs::dsl::*;

    let res = diesel::update(logs)
        .filter(id.eq(log_id))
        .set(last_activity.eq(timestamp))
        .execute(db);

    if res.is_err() {
        return Err(Error::Undefined);
    }

    Ok(())
}

/// Appends a new location.
pub fn add_info(db: &mut SqliteConnection, new_info: NewInfo) -> Result<LogLocationPoint, Error> {
    use schema::info;

    if !new_info.is_valid() {
        return Err(Error::Undefined);
    }

    get_log(db, new_info.log_id).ok_or(Error::NotFound)?;

    let res = diesel::insert_into(info::table)
        .values(&new_info)
        .returning(LogLocationPoint::as_returning())
        .get_result(db);

    let ulp = match res {
        Ok(lp) => lp,
        Err(_) => return Err(Error::Undefined),
    };

    if let Some(ts) = new_info.server_timestamp {
        if set_last_activity(db, new_info.log_id, ts).is_err() {
            return Err(Error::Undefined);
        }
    }

    Ok(ulp)
}

/// Appends a new encrypted location.
pub fn add_info_sec(db: &mut SqliteConnection, new_info: NewInfoSec) -> Result<(), Error> {
    use schema::info_sec;

    get_log(db, new_info.log_id).ok_or(Error::NotFound)?;

    let res = diesel::insert_into(info_sec::table)
        .values(&new_info)
        .execute(db);

    if res.is_err() {
        return Err(Error::Undefined);
    }

    if let Some(ts) = new_info.server_timestamp {
        if set_last_activity(db, new_info.log_id, ts).is_err() {
            return Err(Error::Undefined);
        }
    }

    Ok(())
}

/// Filters the given locations to keep only the last meaningful locations. The
/// definition of "meaningful" is what makes this function important (and
/// currently, it's very naive).
fn filter_last_info(values: Vec<LogLocationPoint>, time_gap_secs: i32) -> Vec<LogLocationPoint> {
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

/// Filters the given locations to keep only the last meaningful locations.
///
/// The locations being encrypted, there's no way for the server to define any
/// "meaningful" way of splitting. Maybe by looking at server timestamps?
fn filter_last_info_sec(
    values: Vec<LogLocationPointSec>,
    time_gap_secs: i32,
) -> Vec<LogLocationPointSec> {
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

/// Gets the last locations for a given log.
///
/// The `cut_last_segment` can be set to true to only keep the last segment.
pub fn get_last_info(
    db: &mut SqliteConnection,
    uid: i32,
    count: i64,
    cut_last_segment: bool,
) -> Option<Vec<LogLocationPoint>> {
    use schema::info::dsl::*;

    let last_pos = info
        .filter(log_id.eq(uid))
        .limit(count)
        .select(LogLocationPoint::as_select())
        // FIXME: this returns the last locations received by the server. If the
        // client sends locations in incorrect order (i.e. more recent first,
        // then some old), this may return unexpected result.
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

/// Gets the last locations for a given encrypted log.
///
/// The `cut_last_segment` is unused and will be removed.
pub fn get_last_info_sec(
    db: &mut SqliteConnection,
    uid: i32,
    count: i64,
    cut_last_segment: bool,
) -> Option<Vec<LogLocationPointSec>> {
    use schema::info_sec::dsl::*;

    let last_pos = info_sec
        .filter(log_id.eq(uid))
        .limit(count)
        .select(LogLocationPointSec::as_select())
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
