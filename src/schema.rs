// @generated automatically by Diesel CLI.

diesel::table! {
    info (id) {
        id -> Integer,
        log_id -> Integer,
        server_timestamp -> Integer,
        device_timestamp -> Integer,
        lat -> Double,
        lon -> Double,
        altitude -> Nullable<Double>,
        speed -> Nullable<Double>,
        direction -> Nullable<Double>,
        accuracy -> Nullable<Double>,
        loc_provider -> Nullable<Text>,
        battery -> Nullable<Double>,
    }
}

diesel::table! {
    info_sec (id) {
        id -> Integer,
        log_id -> Integer,
        server_timestamp -> Integer,
        data -> Binary,
    }
}

diesel::table! {
    logs (id) {
        id -> Integer,
        priv_token -> Nullable<Text>,
        unique_url -> Nullable<Text>,
        last_activity -> Nullable<Integer>,
    }
}

diesel::joinable!(info -> logs (log_id));
diesel::joinable!(info_sec -> logs (log_id));

diesel::allow_tables_to_appear_in_same_query!(info, info_sec, logs,);
