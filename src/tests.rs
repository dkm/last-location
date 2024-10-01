#[cfg(test)]
use super::*;
use ::axum_test::TestServer;
use last_position::{
    delete_log, generate_log_token, generate_new_log, get_log, run_migrations, set_unique_url,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use std::time::{SystemTime, UNIX_EPOCH};

// FIXME should be SetLastLocParams
#[derive(Deserialize, Serialize)]
struct TestForm<'a> {
    pub priv_token: &'a str,
    pub device_timestamp: i32,
    pub lat: f64,
    pub lon: f64,

    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub direction: Option<f64>,

    pub accuracy: Option<f64>,

    pub loc_provider: Option<String>,
    pub battery: Option<f64>,
}

impl<'a> TestForm<'a> {
    pub fn new(priv_token: &'a str, device_ts: i32, lat: f64, lon: f64) -> Self {
        TestForm {
            priv_token,
            device_timestamp: device_ts,
            lat,
            lon,
            altitude: None,
            speed: None,
            direction: None,
            accuracy: None,
            loc_provider: None,
            battery: None,
        }
    }

    pub fn new_full(
        priv_token: &'a str,
        device_ts: i32,
        lat: f64,
        lon: f64,
        alt: Option<f64>,
        spd: Option<f64>,
        dir: Option<f64>,
        acc: Option<f64>,
        loc_prov: Option<String>,
        battery: Option<f64>,
    ) -> Self {
        TestForm {
            priv_token,
            device_timestamp: device_ts,
            lat,
            lon,
            altitude: alt,
            speed: spd,
            direction: dir,
            accuracy: acc,
            loc_provider: loc_prov,
            battery,
        }
    }
}

struct TestData<'a> {
    db_file_name: &'a str,
}

impl<'a> Drop for TestData<'a> {
    fn drop(&mut self) {
        fs::remove_file(self.db_file_name).unwrap();
    }
}

impl<'a> TestData<'a> {
    fn new(db_file_name: &'a str) -> Self {
        if Path::new(db_file_name).try_exists().unwrap() {
            fs::remove_file(&db_file_name).unwrap();
        }
        TestData { db_file_name }
    }
}

#[tokio::test]
async fn simple_location_post_get() {
    let test_db = TestData::new("simple_location_post_get.sqlite");

    let curr_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Can't get epoch")
        .as_secs() as i32;

    let manager = Manager::new(test_db.db_file_name, deadpool_diesel::Runtime::Tokio1);
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
        .form(&TestForm::new(&token, curr_time + 1i32, 32.0f64, 22.0f64))
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
    assert_eq!(json_res[0].device_timestamp, curr_time + 1);
    assert_eq!(json_res[0].lat, 32.0f64);
    assert_eq!(json_res[0].lon, 22.0f64);

    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, curr_time + 2i32, 66.0f64, 77.0f64))
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
    assert_eq!(json_res[0].device_timestamp, curr_time + 2);
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
    assert_eq!(json_res[0].device_timestamp, curr_time + 2);
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
        .form(&TestForm::new(&token, curr_time + 3i32, 88.0f64, 99.0f64))
        .await;
    response.assert_status_ok();
    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, curr_time + 4i32, 11.0f64, 22.0f64))
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
}

#[tokio::test]
async fn simple_invalid_location_post_get() {
    let test_db = TestData::new("simple_invalid_location_post_get.sqlite");

    let manager = Manager::new(test_db.db_file_name, deadpool_diesel::Runtime::Tokio1);
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

    let res = conn
        .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
        .await
        .unwrap();
    assert!(res.is_ok());

    let app = app(pool).await;
    let server = TestServer::new(app).unwrap();

    // no data yet
    let response = server
        .get("/api/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_not_found();

    // Out of range latitude
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, -91f64, 22.0f64))
        .await;
    response.assert_status_bad_request();
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, 91f64, 22.0f64))
        .await;
    response.assert_status_bad_request();

    // Out of range longitude
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, 10f64, -181f64))
        .await;
    response.assert_status_bad_request();
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, 10f64, 181f64))
        .await;
    response.assert_status_bad_request();

    // Out of range longitude and latitude
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, -100f64, -200f64))
        .await;
    response.assert_status_bad_request();
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new(&token, 1i32, 100f64, 300f64))
        .await;
    response.assert_status_bad_request();

    // Out of range altitude
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new_full(
            &token,
            1i32,
            10f64,
            20f64,
            Some(-3000f64),
            None,
            None,
            None,
            None,
            None,
        ))
        .await;
    response.assert_status_bad_request();
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm::new_full(
            &token,
            1i32,
            10f64,
            20f64,
            Some(10000f64 + 1f64), // 1-off
            None,
            None,
            None,
            None,
            None,
        ))
        .await;
    response.assert_status_bad_request();

    // Double check that nothing got added in the log
    let response = server
        .get("/api/get_last_location")
        .add_query_param("url", "something_something")
        .add_query_param("count", "10")
        .await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn test_cut_last_segment_get() {
    let test_db = TestData::new("cut_last_segment.sqlite");

    let manager = Manager::new(test_db.db_file_name, deadpool_diesel::Runtime::Tokio1);
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

    let curr_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Can't get epoch")
        .as_secs() as i32;

    for i in 1..11 {
        let response = server
            .post(&"/api/set_last_location")
            .form(&TestForm::new(&token, curr_time + i, 32.0f64, 22.0f64))
            .await;
        response.assert_status_ok();
    }

    for i in 1..21 {
        let response = server
            .post(&"/api/set_last_location")
            .form(&TestForm::new(
                &token,
                curr_time + i + 24 * 3600,
                32.0f64,
                22.0f64,
            ))
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
}

#[tokio::test]
async fn delete_log_test() {
    let test_db = TestData::new("delete_log.sqlite");

    let manager = Manager::new(test_db.db_file_name, deadpool_diesel::Runtime::Tokio1);
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
}

#[tokio::test]
async fn simple_location_post_get_sec() {
    let test_db = TestData::new("simple_location_post_get_sec.sqlite");

    let manager = Manager::new(test_db.db_file_name, deadpool_diesel::Runtime::Tokio1);
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

    let app = app(pool).await;

    let server = TestServer::new(app).unwrap();

    // no data yet
    let response = server
        .get("/api/s/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_not_found();

    // get/set single/only data
    let response = server
        .post(&"/api/s/set_last_location")
        .form(&SetLastLocSecParams {
            priv_token: token.clone(),
            data: "abcdef".to_string(),
        })
        .await;
    response.assert_status_ok();

    let response = server
        .get("/api/s/get_last_location")
        .add_query_param("uid", "1")
        .await;
    response.assert_status_ok();

    let json_res = response.json::<Vec<LogLocationPointSec>>();
    assert_eq!(json_res.len(), 1);
    assert_eq!(json_res[0].log_id, 1);
    assert_eq!(json_res[0].data.len(), 3usize);

    let response = server
        .post(&"/api/s/set_last_location")
        .form(&SetLastLocSecParams {
            priv_token: token.clone(),
            data: "ab".to_string().repeat(400usize),
        })
        .await;
    response.assert_status_ok();

    let response = server
        .post(&"/api/s/set_last_location")
        .form(&SetLastLocSecParams {
            priv_token: token.clone(),
            data: "ab".to_string().repeat(401usize),
        })
        .await;
    response.assert_status_not_ok();

    // let response = server
    //     .get("/api/get_last_location")
    //     .add_query_param("uid", "1")
    //     .await;
    // response.assert_status_ok();
    // let json_res = response.json::<Vec<LogLocationPoint>>();

    // assert_eq!(json_res.len(), 1);
    // assert_eq!(json_res[0].log_id, 1);
    // assert_eq!(json_res[0].device_timestamp, curr_time + 2);
    // assert_eq!(json_res[0].lat, 66.0f64);
    // assert_eq!(json_res[0].lon, 77.0f64);

    // let response = server
    //     .get("/api/get_last_location")
    //     .add_query_param("url", "something_something")
    //     .await;
    // response.assert_status_not_found();

    // let res = conn
    //     .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
    //     .await
    //     .unwrap();
    // assert!(res.is_ok());

    // let response = server
    //     .get("/api/get_last_location")
    //     .add_query_param("url", "something_something")
    //     .await;
    // response.assert_status_ok();

    // let json_res = response.json::<Vec<LogLocationPoint>>();

    // assert_eq!(json_res.len(), 1);
    // assert_eq!(json_res[0].log_id, 1);
    // assert_eq!(json_res[0].device_timestamp, curr_time + 2);
    // assert_eq!(json_res[0].lat, 66.0f64);
    // assert_eq!(json_res[0].lon, 77.0f64);

    // let res = conn
    //     .interact(|conn| set_unique_url(conn, 1i32, "something_something"))
    //     .await
    //     .unwrap();
    // assert!(res.is_ok());

    // // get set latest data
    // let response = server
    //     .post(&"/api/set_last_location")
    //     .form(&TestForm::new(&token, curr_time + 3i32, 88.0f64, 99.0f64))
    //     .await;
    // response.assert_status_ok();
    // // get set latest data
    // let response = server
    //     .post(&"/api/set_last_location")
    //     .form(&TestForm::new(&token, curr_time + 4i32, 11.0f64, 22.0f64))
    //     .await;
    // response.assert_status_ok();

    // let response = server
    //     .get("/api/get_last_location")
    //     .add_query_param("url", "something_something")
    //     .add_query_param("count", "10")
    //     .await;
    // response.assert_status_ok();

    // let json_res = response.json::<Vec<LogLocationPoint>>();

    // assert_eq!(json_res.len(), 4);
}
