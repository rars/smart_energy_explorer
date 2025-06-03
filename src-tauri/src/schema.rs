// @generated automatically by Diesel CLI.

diesel::table! {
    electricity_consumption (electricity_consumption_id) {
        electricity_consumption_id -> Integer,
        timestamp -> Timestamp,
        energy_consumption_kwh -> Double,
    }
}

diesel::table! {
    electricity_standing_charge (electricity_standing_charge_id) {
        electricity_standing_charge_id -> Integer,
        start_date -> Timestamp,
        standing_charge_pence -> Double,
    }
}

diesel::table! {
    electricity_tariff_plan (tariff_id) {
        tariff_id -> Text,
        plan -> Text,
        effective_date -> Timestamp,
        display_name -> Text,
    }
}

diesel::table! {
    electricity_unit_price (electricity_unit_price_id) {
        electricity_unit_price_id -> Integer,
        price_effective_time -> Timestamp,
        unit_price_pence -> Double,
    }
}

diesel::table! {
    energy_profile (energy_profile_id) {
        energy_profile_id -> Integer,
        name -> Text,
        is_active -> Bool,
        start_date -> Timestamp,
        last_date_retrieved -> Nullable<Timestamp>,
        base_unit -> Text,
    }
}

diesel::table! {
    gas_consumption (gas_consumption_id) {
        gas_consumption_id -> Integer,
        timestamp -> Timestamp,
        energy_consumption_kwh -> Double,
    }
}

diesel::table! {
    gas_standing_charge (gas_standing_charge_id) {
        gas_standing_charge_id -> Integer,
        start_date -> Timestamp,
        standing_charge_pence -> Double,
    }
}

diesel::table! {
    gas_tariff_plan (tariff_id) {
        tariff_id -> Text,
        plan -> Text,
        effective_date -> Timestamp,
        display_name -> Text,
    }
}

diesel::table! {
    gas_unit_price (gas_unit_price_id) {
        gas_unit_price_id -> Integer,
        price_effective_time -> Timestamp,
        unit_price_pence -> Double,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    electricity_consumption,
    electricity_standing_charge,
    electricity_tariff_plan,
    electricity_unit_price,
    energy_profile,
    gas_consumption,
    gas_standing_charge,
    gas_tariff_plan,
    gas_unit_price,
);
