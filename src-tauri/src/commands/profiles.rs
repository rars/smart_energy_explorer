use log::debug;
use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::{
    data::energy_profile::{EnergyProfile, EnergyProfileRepository, SqliteEnergyProfileRepository},
    download::spawn_download_tasks,
    utils::{get_glowmarkt_data_provider, parse_iso_string_to_naive_date},
    AppState,
};

use super::ApiError;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnergyProfileUpdateParam {
    pub energy_profile_id: i32,
    pub is_active: bool,
    pub start_date: String,
}

#[tauri::command]
pub fn get_energy_profiles(app_state: State<'_, AppState>) -> Result<Vec<EnergyProfile>, ApiError> {
    let repository = SqliteEnergyProfileRepository::new(app_state.db.clone());

    repository
        .get_all_energy_profiles()
        .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))
}

#[tauri::command]
pub async fn update_energy_profile_settings(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    energy_profile_updates: Vec<EnergyProfileUpdateParam>,
) -> Result<(), ApiError> {
    let update_settings: Result<Vec<_>, ApiError> = energy_profile_updates
        .iter()
        .map(|x| {
            Ok((
                x.energy_profile_id,
                x.is_active,
                parse_iso_string_to_naive_date(&x.start_date)?,
            ))
        })
        .collect();

    let repository = SqliteEnergyProfileRepository::new(app_state.db.clone());

    for (energy_profile_id, is_active, start) in update_settings? {
        debug!("Updating {}, {}, {}", energy_profile_id, is_active, start);

        let _ = repository.update_energy_profile_settings(
            energy_profile_id,
            is_active,
            start.into(),
        )?;
    }

    let app_state_clone = (*app_state).clone();

    if let Some(data_provider) = get_glowmarkt_data_provider()
        .await
        .map_err(|e| ApiError::Custom(e.to_string()))?
    {
        spawn_download_tasks(app_handle, app_state_clone, data_provider)
            .map_err(|e| ApiError::Custom(e.to_string()))?;
    }

    Ok(())
}
