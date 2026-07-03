use serde::{Deserialize, Serialize};

use crate::{local_state, ApiError, ApiResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MidiScanReportDTO {
    pub watched_directories: Vec<String>,
    pub discovered_files: usize,
    pub registered_files: usize,
}

pub async fn add_watch_directory(directory: String) -> ApiResult<MidiScanReportDTO> {
    add_watch_directories(vec![directory]).await
}

pub async fn add_watch_directories(directories: Vec<String>) -> ApiResult<MidiScanReportDTO> {
    let state = local_state::state().await?;
    let report = state
        .0
        .music
        .local_library
        .add_watch_directories(directories)
        .await
        .map_err(ApiError::from)?;

    Ok(report_to_dto(report))
}

pub async fn list_watch_directories() -> ApiResult<Vec<String>> {
    let state = local_state::state().await?;
    state
        .0
        .music
        .local_library
        .list_watch_directories()
        .await
        .map_err(ApiError::from)
}

pub async fn refresh_watched_directories() -> ApiResult<MidiScanReportDTO> {
    let state = local_state::state().await?;
    let report = state
        .0
        .music
        .local_library
        .refresh_watched_directories()
        .await
        .map_err(ApiError::from)?;

    Ok(report_to_dto(report))
}

pub async fn refresh_if_dirty() -> ApiResult<Option<MidiScanReportDTO>> {
    let state = local_state::state().await?;
    let report = state
        .0
        .music
        .local_library
        .refresh_if_dirty()
        .await
        .map_err(ApiError::from)?;

    Ok(report.map(report_to_dto))
}

fn report_to_dto(report: app::music::MidiScanReport) -> MidiScanReportDTO {
    MidiScanReportDTO {
        watched_directories: report.watched_directories,
        discovered_files: report.discovered_files,
        registered_files: report.registered_files,
    }
}
