// @generated automatically by Diesel CLI.

diesel::table! {
    electricity_consumption (electricity_consumption_id) {
        electricity_consumption_id -> Integer,
        timestamp -> Timestamp,
        energy_consumption_kwh -> Double,
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
    gas_consumption,
);
