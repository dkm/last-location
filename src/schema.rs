// @generated automatically by Diesel CLI.

diesel::table! {
    info (ts) {
        id -> Integer,
        lat -> Double,
        lon -> Double,
        accuracy -> Integer,
        ts -> Timestamp,
    }
}

diesel::table! {
    pilot (id) {
        id -> Integer,
        name -> Nullable<Text>,
    }
}

diesel::joinable!(info -> pilot (id));

diesel::allow_tables_to_appear_in_same_query!(
    info,
    pilot,
);
