use diesel::debug_query;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use last_position::models::UserInfo;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    use last_position::schema::users::dsl::*;

    let limit_count = 40i32;

    let db = &mut establish_connection();

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
