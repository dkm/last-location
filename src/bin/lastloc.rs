use clap::{Arg, ArgAction, ArgMatches, Command};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use last_position::{
    add_info, add_info_sec, delete_log, generate_log_token, generate_new_log, get_all_logs, init,
    models::{LogLocationPoint, LogLocationPointSec, NewInfo, NewInfoSec},
    run_migrations, set_unique_url,
};

pub fn establish_connection(db_url: &str) -> SqliteConnection {
    //let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

fn do_gen_priv_token(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let uid = matches
        .get_one::<String>("log-id")
        .unwrap()
        .parse::<i32>()
        .expect("Not an i32");
    generate_log_token(db, uid).expect("Error generating priv");
}

fn do_create_log(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let no_token_url = !matches.get_one::<bool>("no-token-url").unwrap();

    generate_new_log(db, no_token_url).expect("Error creating new log");
}

fn do_delete_log(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let uid = matches
        .get_one::<String>("log-id")
        .unwrap()
        .parse::<i32>()
        .expect("Not an i32");

    delete_log(db, uid).expect("Error deleting log");
}

fn do_list_logs(db_url: &str, _matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let all_logs = get_all_logs(db);
    match all_logs {
        None => println!("No log"),
        Some(v) => {
            for log in v {
                println!("{}", log);
            }
        }
    }
}

fn do_set_unique_url(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let url = matches.get_one::<String>("url").unwrap();
    let log_id = matches
        .get_one::<String>("log-id")
        .unwrap()
        .parse::<i32>()
        .expect("Not an i32");
    set_unique_url(db, log_id, url).expect("Error setting url");
}

// During testing, server timestamp can be forced to specific value.
fn get_time() -> i32 {
    if let Ok(v) = env::var("LASTLOC_MOCK_SERVER_TIME") {
        println!("Mock time {}", v.parse::<i32>().expect("Not an i32"));
        v.parse::<i32>().expect("Not an i32")
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't get epoch")
            .as_secs() as i32
    }
}

fn do_expire_logs(db_url: &str, matches: &ArgMatches) {
    let limit_count = matches
        .get_one::<String>("max-count")
        .unwrap()
        .parse::<i32>()
        .expect("Not an i32");
    let secure = matches.get_flag("secure");

    let limit_lifetime = matches
        .get_one::<String>("max-lifetime")
        .and_then(|v| Some(v.parse::<i32>().expect("not an i32")));

    let cur_ts = get_time();

    let db = &mut establish_connection(db_url);

    let all_logs = get_all_logs(db).expect("failed to get logs, FIXME error handling");

    // FIXME both case are identical, only the types change. Make it generic.
    for log in all_logs {
        if !secure {
            use last_position::schema::info::dsl::*;

            let old_count = info
                .filter(log_id.eq(log.id))
                .count()
                .first::<i64>(db)
                .expect("Error counting existing log points");

            let res = if let Some(limit_lt) = limit_lifetime {
                let oldest_ts = cur_ts - limit_lt;
                info.select(server_timestamp)
                    .filter(log_id.eq(log.id).and(server_timestamp.ge(oldest_ts)))
                    .order(server_timestamp.desc())
                    .limit(limit_count as i64)
                    .load::<i32>(db)
                    .expect("can't load last entries to keep")
            } else {
                info.select(server_timestamp)
                    .filter(log_id.eq(log.id))
                    .order(server_timestamp.desc())
                    .limit(limit_count as i64)
                    .load::<i32>(db)
                    .expect("can't load last entries to keep")
            };

            if res.is_empty() {
                continue;
            }

            let ts_limit = res[(res.len() - 1) as usize];

            let to_expire = info.filter(server_timestamp.lt(ts_limit));

            // Keeping this for future "--verbose"
            // let sql = debug_query::<diesel::sqlite::Sqlite, _>(&to_expire).to_string();
            // println!("SQL: {}", sql);

            let expire_count = to_expire
                .count()
                .first::<i64>(db)
                .expect("Error counting rows to expire");

            println!(
                "Log: {}, pre-count: {}, to expire: {}",
                log.id, old_count, expire_count
            );

            let _ = diesel::delete(to_expire).execute(db);

            // let new_count = info
            //     .filter(log_id.eq(log.id))
            //     .count()
            //     .first::<i64>(db)
            //     .expect("Error counting existing measures");
        } else {
            use last_position::schema::info_sec::dsl::*;

            let old_count = info_sec
                .filter(log_id.eq(log.id))
                .count()
                .first::<i64>(db)
                .expect("Error counting existing log points");

            let res = if let Some(limit_lt) = limit_lifetime {
                let oldest_ts = cur_ts - limit_lt;
                info_sec.select(server_timestamp)
                    .filter(log_id.eq(log.id).and(server_timestamp.ge(oldest_ts)))
                    .order(server_timestamp.desc())
                    .limit(limit_count as i64)
                    .load::<i32>(db)
                    .expect("can't load last entries to keep")
            } else {
                info_sec.select(server_timestamp)
                    .filter(log_id.eq(log.id))
                    .order(server_timestamp.desc())
                    .limit(limit_count as i64)
                    .load::<i32>(db)
                    .expect("can't load last entries to keep")
            };

            if res.is_empty() {
                continue;
            }

            let ts_limit = res[(res.len() - 1) as usize];

            let to_expire = info_sec.filter(server_timestamp.lt(ts_limit));

            // Keeping this for future "--verbose"
            // let sql = debug_query::<diesel::sqlite::Sqlite, _>(&to_expire).to_string();
            // println!("SQL: {}", sql);

            let expire_count = to_expire
                .count()
                .first::<i64>(db)
                .expect("Error counting rows to expire");

            println!(
                "Log: {}, pre-count: {}, to expire: {}",
                log.id, old_count, expire_count
            );
        }
    }
}

fn do_init(db_url: &str, _matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    run_migrations(db).expect("Can't init/run migrations")
}

fn do_add_to_log(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let secure = matches.get_flag("secure");

    let data_path = matches.get_one::<String>("data").expect("can't be missing");
    let data_str = std::fs::read_to_string(data_path).expect("Unable to read file");
    if !secure {
        let all_data: Vec<NewInfo> =
            serde_json::from_str(&data_str).expect("JSON does not have correct format.");
        for ni in all_data {
            add_info(db, ni).expect("error adding location");
        }
    } else {
        let all_data: Vec<NewInfoSec> =
            serde_json::from_str(&data_str).expect("JSON does not have correct format.");
        for ni in all_data {
            add_info_sec(db, ni).expect("error adding location");
        }
    }
}

fn do_dump_logs(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);

    let secure = matches.get_flag("secure");
    let output_path = matches
        .get_one::<String>("output")
        .expect("can't be missing");

    if !secure {
        use last_position::schema::info::dsl::*;
        let last_pos = info
            .select(LogLocationPoint::as_select())
            .order(server_timestamp.desc())
            .load(db)
            .expect("Error querying db");

        fs::write(output_path, serde_json::to_string(&last_pos).unwrap())
            .expect("Error writing to file");
    } else {
        use last_position::schema::info_sec::dsl::*;
        let last_pos = info_sec
            .select(LogLocationPointSec::as_select())
            .order(id.desc())
            .load(db)
            .expect("Error querying db");
        fs::write(output_path, serde_json::to_string(&last_pos).unwrap())
            .expect("Error writing to file");
    }
}

fn do_list_locations(db_url: &str, matches: &ArgMatches) {
    let db = &mut establish_connection(db_url);
    let logid = matches
        .get_one::<String>("log-id")
        .unwrap()
        .parse::<i32>()
        .expect("Not an i32");

    let limit_count = matches
        .get_one::<String>("max-count")
        .unwrap()
        .parse::<i64>()
        .expect("Not an i64");

    let secure = matches.get_flag("secure");

    if !secure {
        use last_position::schema::info::dsl::*;
        let last_pos = info
            .filter(log_id.eq(logid))
            .limit(limit_count)
            .select(LogLocationPoint::as_select())
            .order(server_timestamp.desc())
            .load(db)
            .expect("Error querying db");

        for pos in last_pos {
            println!("- {}", pos);
        }
    } else {
        use last_position::schema::info_sec::dsl::*;
        let last_pos = info_sec
            .filter(log_id.eq(logid))
            .limit(limit_count)
            .select(LogLocationPointSec::as_select())
            .order(id.desc())
            .load(db)
            .expect("Error querying db");

        for pos in last_pos {
            println!("- [crypt] {}", pos);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("lastloc")
        .version("0.1")
        .author("Marc Poulhiès <dkm@kataplop.net>")
        .about("last-location cli")
        .arg(
            Arg::new("sqlite-db")
                .long("sqlite-db")
                .default_value("info.sqlite"),
        )
        .subcommand(Command::new("init"))
        .subcommand(
            Command::new("expire-logs")
                .arg(Arg::new("max-lifetime").long("max-lifetime"))
                .arg(Arg::new("max-count").long("max-count"))
                .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("create-log").arg(
                Arg::new("no-token-url")
                    .long("no-token-url")
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("add-to-log")
                .arg(Arg::new("data").long("data"))
                .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("dump-logs")
                .arg(Arg::new("output").long("output"))
                .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue)),
        )
        .subcommand(
            Command::new("delete-log").arg(Arg::new("log-id").long("log-id").required(true)),
        )
        .subcommand(Command::new("list-logs"))
        .subcommand(
            Command::new("list-locations")
                .arg(Arg::new("log-id").long("log-id").required(true))
                .arg(Arg::new("max-count").long("max-count").default_value("10"))
                .arg(Arg::new("secure").long("secure").action(ArgAction::SetTrue)),
        )
        .subcommand(Command::new("gen-priv-token").arg(Arg::new("log-id").long("log-id")))
        .subcommand(
            Command::new("set-unique-url")
                .arg(Arg::new("url").long("url").required(true))
                .arg(Arg::new("log-id").long("log-id").required(true)),
        )
        .get_matches();

    let sql_db = matches
        .get_one::<String>("sqlite-db")
        .ok_or("can't happen")?;

    let db = &mut establish_connection(sql_db);
    init(db).unwrap();

    match matches.subcommand() {
        Some(("init", sub_matches)) => do_init(sql_db, sub_matches),
        Some(("expire-logs", sub_matches)) => do_expire_logs(sql_db, sub_matches),
        Some(("gen-priv-token", sub_matches)) => do_gen_priv_token(sql_db, sub_matches),
        Some(("set-unique-url", sub_matches)) => do_set_unique_url(sql_db, sub_matches),
        Some(("create-log", sub_matches)) => do_create_log(sql_db, sub_matches),
        Some(("delete-log", sub_matches)) => do_delete_log(sql_db, sub_matches),
        Some(("list-logs", sub_matches)) => do_list_logs(sql_db, sub_matches),
        Some(("list-locations", sub_matches)) => do_list_locations(sql_db, sub_matches),
        Some(("add-to-log", sub_matches)) => do_add_to_log(sql_db, sub_matches),
        Some(("dump-logs", sub_matches)) => do_dump_logs(sql_db, sub_matches),

        _ => println!("Wooops"),
    }

    Ok(())
}
