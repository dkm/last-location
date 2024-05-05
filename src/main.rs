#[cfg(test)]
mod tests;

use axum::{
    extract::Path,
    extract::{Query, State},
    http::StatusCode,
    response::{Json, Redirect},
    routing::{get, post},
    Form, Router,
};

use diesel::SqliteConnection;
use last_position::{
    get_user_from_token,
    models::{NewInfo, UserLocationPoint, UserInfo},
    run_migrations,
};

use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use serde::{de, Deserialize, Deserializer};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{event, Level};

use deadpool_diesel::sqlite::{Manager, Pool};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Placeholder struct to store app state.
#[derive(Clone)]
struct S {
    pool: Pool,
}

async fn app(pool: Pool) -> Router {
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| run_migrations(conn))
        .await
        .unwrap()
        .unwrap();

    Router::new()
        .route("/s/:uniq_url", get(get_stable_infopage))
        .route("/api/new", post(create_new_user))
        .route("/api/get_last_location", get(get_last_location))
        .route("/api/set_last_location", post(set_last_location))
        .nest_service("/", ServeDir::new("static"))
        .with_state(S { pool })
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

    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .expect("Can't create a pool for Sqlite db");

    let app = app(pool).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
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

async fn get_stable_infopage(
    Path(uniq_url): Path<String>,
) -> Result<Redirect, (StatusCode, String)> {
    event!(Level::TRACE, "stable {}", uniq_url);
    Ok(Redirect::temporary(&format!("/?u={uniq_url}")))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CreateNewUserParams {
    #[serde(default)]
    req_url: String,

    #[serde(default)]
    name: String,
}

async fn create_new_user(
    State(s): State<S>,
    Form(params): Form<CreateNewUserParams>,
) -> Result<Json<UserInfo>, (StatusCode, String)> {
    event!(Level::TRACE, "create_new_user {} {} ", params.req_url, params.name);
    let conn = s.pool.get().await.map_err(internal_error)?;

    // Oh, this is ugly O_o

    let uinfo = conn
        .interact(move |conn| last_position::create_user(conn, &params.name))
        .await
        .unwrap();
    if let Ok(uinfo) = uinfo {
        event!(Level::TRACE, " got a user {} ", uinfo);
        let res = conn
            .interact(move |conn| last_position::set_unique_url(conn, uinfo.id, &params.req_url))
            .await
            .unwrap();
        event!(Level::TRACE, " set unique  {} ", res.is_ok());

        let res = conn
            .interact(move |conn| last_position::generate_user_token(conn, uinfo.id))
            .await
            .unwrap();
        event!(Level::TRACE, " generated a token {} ", res.is_ok());

        if res.is_ok() {
            let r = conn
                .interact(move |conn| last_position::get_user_from_id(conn, uinfo.id))
                .await
                .unwrap().unwrap();

            event!(Level::TRACE, " get user again  {} ", r);
            return Ok(Json(r));
        }
        return Err((StatusCode::NOT_FOUND, "No match".to_string()));
    } else {
        return Err((StatusCode::NOT_FOUND, "No match".to_string()));
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GetLastLocParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    uid: Option<i32>,

    #[serde(default, deserialize_with = "empty_string_as_none")]
    url: Option<String>,

    count: Option<i64>,
}

async fn get_last_location(
    Query(params): Query<GetLastLocParams>,
    State(s): State<S>,
) -> Result<Json<Vec<UserLocationPoint>>, (StatusCode, String)> {
    let conn = s.pool.get().await.map_err(internal_error)?;

    if params.uid.is_some() && params.url.is_some()
        || (params.uid.is_none() && params.url.is_none())
    {
        return Err((StatusCode::NOT_FOUND, "No match".to_string()));
    }

    let uid = if let Some(u) = params.uid {
        u
    } else {
        let url = params.url.unwrap();

        let uinfo = conn
            .interact(move |conn| last_position::get_user_from_url(conn, &url))
            .await
            .unwrap();
        if let Some(uinfo) = uinfo {
            uinfo.id
        } else {
            return Err((StatusCode::NOT_FOUND, "No match".to_string()));
        }
    };

    let count = match params.count {
        Some(c) => c,
        None => 1,
    };

    event!(Level::TRACE, "Get {}", uid);
    let res = conn
        .interact(move |conn| last_position::get_last_info(conn, uid, count))
        .await
        .map_err(internal_error)?;

    if res.is_some() {
        Ok(Json(res.unwrap()))
    } else {
        Err((StatusCode::NOT_FOUND, "No match".to_string()))
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SetLastLocParams {
    // this is the only diff with NewInfo
    pub priv_token: String,

    pub device_timestamp: i32,
    pub server_timestamp: Option<i32>,

    pub lat: f64,
    pub lon: f64,

    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

impl SetLastLocParams {
    pub fn to_newinfo(&self, db: &mut SqliteConnection) -> Option<NewInfo> {
        let uinfo = get_user_from_token(db, &self.priv_token);

        match uinfo {
            Some(uinfo) => Some(NewInfo {
                user_id: uinfo.id,

                device_timestamp: self.device_timestamp,
                server_timestamp: self.server_timestamp,
                lat: self.lat,
                lon: self.lon,
                altitude: self.altitude,
                speed: self.speed,
                direction: self.direction,
                accuracy: self.accuracy,
                loc_provider: self.loc_provider.clone(),
                battery: self.battery,
            }),
            None => None,
        }
    }
}

async fn set_last_location(
    State(s): State<S>,
    Form(new_info): Form<SetLastLocParams>,
) -> Result<Json<UserLocationPoint>, (StatusCode, String)> {
    let conn = s.pool.get().await.map_err(internal_error)?;

    let pinfo = conn
        .interact(move |conn| new_info.to_newinfo(conn))
        .await
        .map_err(internal_error)?;

    if pinfo.is_none() {
        return Err((StatusCode::NOT_FOUND, "No match".to_string()));
    }

    let mut pinfo = pinfo.unwrap();

    pinfo.server_timestamp = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't get epoch")
            .as_secs() as i32,
    );
    event!(Level::TRACE, "Set {:?}", pinfo);

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
