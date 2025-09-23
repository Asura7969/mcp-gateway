use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String,
    pub active_connections: usize,
    pub shutdown_in_progress: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Get system status endpoint
pub async fn get_system_status(
    State(state): State<AppState>,
) -> Result<Json<SystemStatus>, StatusCode> {
    let status = SystemStatus {
        status: if state.shutdown_coordinator.is_shutdown_in_progress() {
            "shutting_down".to_string()
        } else {
            "running".to_string()
        },
        active_connections: state.shutdown_coordinator.active_connections_count(),
        shutdown_in_progress: state.shutdown_coordinator.is_shutdown_in_progress(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(status))
}
