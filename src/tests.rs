#[cfg(test)]
use super::*;
use ::axum_test::TestServer;
use last_position::{
    delete_log, generate_log_token, generate_new_log, get_log, run_migrations, set_unique_url,
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
        .interact(|conn| generate_new_log(conn, false))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| generate_log_token(conn, 1i32))
        .await
        .unwrap();
    assert!(res.is_ok());

    let token = res.unwrap();

    let res = conn.interact(|conn| get_log(conn, 1i32)).await.unwrap();
    assert!(res.is_some());

    let res = conn.interact(|conn| get_log(conn, 2i32)).await.unwrap();
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

    let json_res = response.json::<Vec<LogLocationPoint>>();
    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].log_id, 1);
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
    let json_res = response.json::<Vec<LogLocationPoint>>();

    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].log_id, 1);
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

    let json_res = response.json::<Vec<LogLocationPoint>>();

    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].log_id, 1);
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

    let json_res = response.json::<Vec<LogLocationPoint>>();

    assert_eq!(json_res.len(), 4);

    fs::remove_file(&db_url).unwrap();
}

#[tokio::test]
async fn test_cut_last_segment_get() {
    let db_url = "cut_last_segment.sqlite";
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
        .interact(|conn| generate_new_log(conn, false))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| generate_log_token(conn, 1i32))
        .await
        .unwrap();
    assert!(res.is_ok());

    let token = res.unwrap();

    let res = conn.interact(|conn| get_log(conn, 1i32)).await.unwrap();
    assert!(res.is_some());

    let res = conn.interact(|conn| get_log(conn, 2i32)).await.unwrap();
    assert!(res.is_none());

    let app = app(pool).await;

    let server = TestServer::new(app).unwrap();

    for i in 1..11 {
        let response = server
            .post(&"/api/set_last_location")
            .form(&TestForm {
                priv_token: &token,
                device_timestamp: i,
                lat: 32.0f64,
                lon: 22.0f64,
            })
            .await;
        response.assert_status_ok();
    }

    for i in 1..21 {
        let response = server
            .post(&"/api/set_last_location")
            .form(&TestForm {
                priv_token: &token,
                device_timestamp: i + 24 * 3600,
                lat: 32.0f64,
                lon: 22.0f64,
            })
            .await;
        response.assert_status_ok();
    }

    let res = conn
        .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
        .await
        .unwrap();
    assert!(res.is_ok());

    // Requesting 30 values (there are enough values) but also trying to cut
    // segments based on time gaps. Should only return 20 values.
    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .add_query_param("cut_last_segment", "true")
        .add_query_param("count", "30")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<LogLocationPoint>>();
    assert_eq!(json_res.len(), 20);

    // Requesting 30 values without filtering out anything.
    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .add_query_param("cut_last_segment", "false")
        .add_query_param("count", "30")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<LogLocationPoint>>();
    assert_eq!(json_res.len(), 30);

    // Requestingr 50 values without filtering out anything.
    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .add_query_param("cut_last_segment", "false")
        .add_query_param("count", "50")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<LogLocationPoint>>();
    assert_eq!(json_res.len(), 30);
    fs::remove_file(&db_url).unwrap();
}

#[tokio::test]
async fn delete_log_test() {
    let db_url = "delete_log.sqlite";
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
        .interact(|conn| generate_new_log(conn, false))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn
        .interact(|conn| generate_new_log(conn, false))
        .await
        .unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_log(conn, 1)).await.unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_log(conn, 1)).await.unwrap();
    assert!(res.is_err());

    let res = conn.interact(|conn| delete_log(conn, 3)).await.unwrap();
    assert!(res.is_err());

    let res = conn.interact(|conn| delete_log(conn, 2)).await.unwrap();
    assert!(res.is_ok());

    let res = conn.interact(|conn| delete_log(conn, 2)).await.unwrap();
    assert!(res.is_err());
    fs::remove_file(&db_url).unwrap();
}
