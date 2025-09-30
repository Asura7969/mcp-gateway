use crate::state::AppState;
use crate::utils::get_china_time;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Get system status endpoint
pub async fn get_system_status(
    State(_state): State<AppState>,
) -> Result<Json<SystemStatus>, StatusCode> {
    let status = SystemStatus {
        status: "running".to_string(),
        timestamp: get_china_time(),
    };

    Ok(Json(status))
}
