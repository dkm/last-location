#[cfg(test)]
use super::*;
use ::axum_test::TestServer;
use last_position::{create_user, get_user, run_migrations};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct TestForm {
    pub user_id: i32,
    pub device_timestamp: i32,
    pub lat: f64,
    pub lon: f64,
}

#[tokio::test]
async fn simple_location_post_get() {
    let db_url = "simple_location_post_get.sqlite";
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
            user_id: 1i32,
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

    let json_res = response.json::<UserLocationPoint>();

    assert_eq!(json_res.user_id, 1);
    assert_eq!(json_res.device_timestamp, 1);
    assert_eq!(json_res.lat, 32.0f64);
    assert_eq!(json_res.lon, 22.0f64);

    // get set latest data
    let response = server
        .post(&"/api/set_last_location")
        .form(&TestForm {
            user_id: 1i32,
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

    let json_res = response.json::<UserLocationPoint>();

    assert_eq!(json_res.user_id, 1);
    assert_eq!(json_res.device_timestamp, 2);
    assert_eq!(json_res.lat, 66.0f64);
    assert_eq!(json_res.lon, 77.0f64);
}
