// @generated automatically by Diesel CLI.

diesel::table! {
    info (id) {
        pilot_id -> Integer,
        id -> Integer,
        ts -> Integer,
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
    pilot (id) {
        id -> Integer,
        name -> Nullable<Text>,
    }
}

diesel::joinable!(info -> pilot (pilot_id));

diesel::allow_tables_to_appear_in_same_query!(
    info,
    pilot,
);
