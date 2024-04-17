use clap::{Arg, ArgGroup, ArgMatches, Command};
use diesel::debug_query;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use last_position::{
    generate_user_token,
    create_user,
    set_unique_url,
    models::UserInfo
};
use std::env;

pub fn establish_connection(db_url: &str) -> SqliteConnection {
    //let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}

fn do_gen_priv_token(db_url : &str, matches: &ArgMatches) {
    let mut db = &mut establish_connection(db_url);
    generate_user_token(&mut db, 1).expect("Error generatng priv");
}

fn do_create_user(db_url : &str, matches: &ArgMatches) {
    let mut db = &mut establish_connection(db_url);
    let name = matches.get_one::<String>("name").unwrap();

    create_user(&mut db, name).expect("Error creating user");
}

fn do_set_unique_url(db_url : &str, matches: &ArgMatches) {
    let mut db = &mut establish_connection(db_url);
    let url = matches.get_one::<String>("url").unwrap();
    let user_id = matches.get_one::<String>("user-id").unwrap().parse::<i32>().expect("Not an i32");
    set_unique_url(&mut db, user_id, url).expect("Error setting url");
}

fn do_expire(db_url : &str, matches: &ArgMatches) {

    use last_position::schema::users::dsl::*;

    let limit_count = 40i32;

    let db = &mut establish_connection(db_url);

    let all_users = users
        .select(UserInfo::as_select())
        .load(db)
        .expect("Error finding all users");

    for user in all_users {
        use last_position::schema::info::dsl::*;
        let old_count = info
            .filter(user_id.eq(user.id))
            .count()
            .first::<i64>(db)
            .expect("Error counting existing measures");

        if old_count < (limit_count as i64) {
            continue;
        }

        let to_keep = info
            .select(server_timestamp)
            .filter(user_id.eq(user.id))
            .order(server_timestamp.desc())
            .limit(limit_count as i64);

        let res = to_keep.load::<i32>(db).expect("Can't load last records");
        let ts_limit = res[(limit_count - 1) as usize];

        let to_expire = info.filter(server_timestamp.le(ts_limit));

        let sql = debug_query::<diesel::sqlite::Sqlite, _>(&to_expire).to_string();
        let expire_count = to_expire
            .count()
            .first::<i64>(db)
            .expect("Error counting rows to expire");

        println!("SQL: {}", sql);
        println!(
            "User: {}, all count: {}, to expire: {}",
            user.id, old_count, expire_count
        );

        let _ = diesel::delete(to_expire).execute(db);

        let new_count = info
            .filter(user_id.eq(user.id))
            .count()
            .first::<i64>(db)
            .expect("Error counting existing measures");

        println!("User: {}, new count: {}", user.id, new_count);
    }
}

fn main() {
    let matches = Command::new("lastloc")
        .version("0.1")
        .author("Marc Poulhiès <dkm@kataplop.net>")
        .about("last-location cli")
        .arg(
            Arg::new("sqlite-db")
                .long("sqlite-db")
                .default_value("info.sqlite"),
        )
        .subcommand(Command::new("expire"))
        .subcommand(Command::new("create-user")
            .arg(
                Arg::new("name")
                    .long("name")
            ))
        .subcommand(Command::new("gen-priv-token"))
        .subcommand(Command::new("set-unique-url")
                    .arg(
                        Arg::new("url")
                            .long("url")
                    )
                    .arg(
                        Arg::new("user-id")
                            .long("user-id")
                    )
)
        .get_matches();

    let sql_db = matches
        .get_one::<String>("sqlite-db")
        .expect("can't be missing");

    match matches.subcommand() {
        Some(("expire", sub_matches)) => do_expire(&sql_db, sub_matches),
        Some(("gen-priv-token", sub_matches)) => do_gen_priv_token(&sql_db, sub_matches),
        Some(("set-unique-url", sub_matches)) => do_set_unique_url(&sql_db, sub_matches),
        Some(("create-user", sub_matches)) => do_create_user(&sql_db, sub_matches),
        _ => println!("Wooops"),
    }
}
