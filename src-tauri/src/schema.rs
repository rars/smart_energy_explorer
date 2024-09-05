// @generated automatically by Diesel CLI.

diesel::table! {
    electricity_consumption (electricity_consumption_id) {
        electricity_consumption_id -> Integer,
        timestamp -> Timestamp,
        energy_consumption_kwh -> Double,
    }
}

diesel::table! {
    energy_profile (energy_profile_id) {
        energy_profile_id -> Integer,
        name -> Text,
        is_active -> Bool,
        start_date -> Timestamp,
        last_date_retrieved -> Nullable<Timestamp>,
    }
}

diesel::table! {
    gas_consumption (gas_consumption_id) {
        gas_consumption_id -> Integer,
        timestamp -> Timestamp,
        energy_consumption_m3 -> Double,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    electricity_consumption,
    energy_profile,
    gas_consumption,
);
