use axum::Form;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::Query;
use serde::{de, Deserialize};

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

use last_position::models::{NewInfo, UserLocationPoint};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "last_position=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    // set up connection pool
    let manager = deadpool_diesel::sqlite::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    // run the migrations on server startup
    {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .unwrap()
            .unwrap();
    }

    let app = Router::new()
        .route("/api/get_last_location", get(get_last_location))
        .route("/api/set_last_location", post(set_last_location))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn set_last_location(
    State(pool): State<deadpool_diesel::sqlite::Pool>,
    Form(new_info): Form<NewInfo>,
) -> Result<Json<UserLocationPoint>, (StatusCode, String)> {
    let mut pinfo: NewInfo = new_info.clone();

    pinfo.server_timestamp = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't get epoch")
            .as_secs() as i32,
    );

    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            last_position::add_info(conn, pinfo)
        })
        .await
        .unwrap()
        .unwrap();
        // .map_err(internal_error)?
        // .map_err(internal_error)?;
    Ok(Json(res))
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct GetLastLocParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    uid: Option<i32>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => std::str::FromStr::from_str(s)
            .map_err(de::Error::custom)
            .map(Some),
    }
}

async fn get_last_location(
    Query(params): Query<GetLastLocParams>,
    State(pool): State<deadpool_diesel::sqlite::Pool>,
) -> Result<Json<Option<UserLocationPoint>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let uid = params.uid.unwrap();
    let res = conn
        .interact(move |conn| {
            last_position::get_last_info(conn, uid)
        })
        .await.unwrap();
        // .map_err(internal_error)?
        // .map_err(internal_error)?;
    Ok(Json(res))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
