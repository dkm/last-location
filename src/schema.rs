// @generated automatically by Diesel CLI.

diesel::table! {
    info (id) {
        id -> Integer,
        user_id -> Integer,
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
        user_id -> Integer,
        server_timestamp -> Integer,
        data -> Binary,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Nullable<Text>,
        priv_token -> Nullable<Text>,
        unique_url -> Nullable<Text>,
    }
}

diesel::joinable!(info -> users (user_id));
diesel::joinable!(info_sec -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(info, info_sec, users,);
