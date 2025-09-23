use crate::models::EndpointMetrics;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};

/// Get metrics for all endpoints
///
/// Returns a list of metrics for all endpoints in the system.
/// This endpoint is used by the dashboard to display aggregate metrics.
pub async fn get_all_endpoint_metrics(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<EndpointMetrics>>, (StatusCode, String)> {
    match app_state.endpoint_service.get_all_endpoint_metrics().await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => {
            tracing::error!("Failed to get all endpoint metrics: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}
