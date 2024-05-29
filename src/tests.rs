#[cfg(test)]
use super::*;
use ::axum_test::TestServer;
use last_position::{
    create_user, delete_user, generate_user_token, get_user, run_migrations, set_unique_url,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Deserialize, Serialize)]
struct TestForm<'a> {
    pub priv_token: &'a str,
    pub device_timestamp: i32,
    pub lat: f64,
    pub lon: f64,
}

#[tokio::test]
async fn simple_location_post_get() {
    let db_url = "simple_location_post_get.sqlite";
    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    let conn = pool.get().await.unwrap();
    let res = conn.interact(|conn| run_migrations(conn)).await;
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| create_user(conn, "sample_user"))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| generate_user_token(conn, 1i32))
        .await
        .unwrap();
    assert!(res.is_ok());

    let token = res.unwrap();

    let res = conn.interact(|conn| get_user(conn, 1i32)).await.unwrap();
    assert!(res.is_some());

    let res = conn.interact(|conn| get_user(conn, 2i32)).await.unwrap();
    assert!(res.is_none());

    let app = app(pool).await;

    let server = TestServer::new(app).unwrap();

    // no data yet
    let response = server
        .get("/api/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_not_found();

    // get/set single/only data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm {
            priv_token: &token,
            device_timestamp: 1i32,
            lat: 32.0f64,
            lon: 22.0f64,
        })
        .await;
    response.assert_status_ok();

    let response = server
        .get("/api/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<UserLocationPoint>>();
    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].user_id, 1);
    assert_eq!(json_res[0].device_timestamp, 1);
    assert_eq!(json_res[0].lat, 32.0f64);
    assert_eq!(json_res[0].lon, 22.0f64);

    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm {
            priv_token: &token,
            device_timestamp: 2i32,
            lat: 66.0f64,
            lon: 77.0f64,
        })
        .await;
    response.assert_status_ok();

    let response = server
        .get("/api/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_ok();
    let json_res = response.json::<Vec<UserLocationPoint>>();

    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].user_id, 1);
    assert_eq!(json_res[0].device_timestamp, 2);
    assert_eq!(json_res[0].lat, 66.0f64);
    assert_eq!(json_res[0].lon, 77.0f64);

    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .await;
    response.assert_status_not_found();

    let res = conn
        .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
        .await
        .unwrap();
    assert!(res.is_ok());

    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<UserLocationPoint>>();

    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].user_id, 1);
    assert_eq!(json_res[0].device_timestamp, 2);
    assert_eq!(json_res[0].lat, 66.0f64);
    assert_eq!(json_res[0].lon, 77.0f64);

    let res = conn
        .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
        .await
        .unwrap();
    assert!(res.is_ok());

    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm {
            priv_token: &token,
            device_timestamp: 3i32,
            lat: 88.0f64,
            lon: 99.0f64,
        })
        .await;
    response.assert_status_ok();
    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm {
            priv_token: &token,
            device_timestamp: 4i32,
            lat: 11.0f64,
            lon: 22.0f64,
        })
        .await;
    response.assert_status_ok();

    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .add_query_param("count", "10")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<UserLocationPoint>>();

    assert_eq!(json_res.len(), 4);
}

#[tokio::test]
async fn delete_user_test() {
    let db_url = "delete_user.sqlite";
    if Path::new(&db_url).try_exists().unwrap() {
        fs::remove_file(&db_url).unwrap();
    }

    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::sqlite::Pool::builder(manager)
        .build()
        .unwrap();

    let conn = pool.get().await.unwrap();
    let res = conn.interact(|conn| run_migrations(conn)).await;
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| create_user(conn, "sample_user"))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| create_user(conn, "sample_user2"))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_user(conn, 1)).await.unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_user(conn, 1)).await.unwrap();
    assert!(res.is_err());

    let res = conn.interact(|conn| delete_user(conn, 3)).await.unwrap();
    assert!(res.is_err());

    let res = conn.interact(|conn| delete_user(conn, 2)).await.unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_user(conn, 2)).await.unwrap();
    assert!(res.is_err());
}
