#[cfg(test)]
mod tests;

use axum::extract::Query;
use axum::Form;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use serde::{de, Deserialize, Deserializer};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{event, Level};

use deadpool_diesel::sqlite::{Manager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use last_position::models::{NewInfo, UserLocationPoint};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

async fn app(pool: Pool) -> Router {
    // run the migrations on server startup
    {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .unwrap()
            .unwrap();
    }

    Router::new()
        .route("/api/get_last_location", get(get_last_location))
        .route("/api/set_last_location", post(set_last_location))
        //        .nest_service("/", ServeDir::new("static"))
        .with_state(pool)
    //        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "last_position=trace,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    // set up connection pool
    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    let app = app(pool).await;

    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // tracing::debug!("listening on {}", addr);
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
    // tracing_subscriber::registry()
    // .with(
    //     tracing_subscriber::EnvFilter::try_from_default_env()
    //         .unwrap_or_else(|_| "last-position=debug,tower_http=debug".into()),
    // )
    // .with(tracing_subscriber::fmt::layer())
    // .init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetLastLocParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    uid: Option<i32>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
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
    event!(Level::TRACE, "Get {}", uid);
    let res = conn
        .interact(move |conn| last_position::get_last_info(conn, uid))
        .await
        .map_err(internal_error)?;

    if res.is_some() {
        Ok(Json(res))
    } else {
        Err((StatusCode::NOT_FOUND, "No match".to_string()))
    }
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
    event!(Level::TRACE, "Set {:?}", pinfo);

    let conn = pool.get().await.map_err(internal_error)?;

    let res = conn
        .interact(|conn| last_position::add_info(conn, pinfo))
        .await
        .map_err(internal_error)?;

    match res {
        Err(_) => Err((StatusCode::NOT_FOUND, "No match".to_string())),
        Ok(r) => Ok(Json(r)),
    }
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
