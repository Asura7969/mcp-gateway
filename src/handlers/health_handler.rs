use crate::models::DbPool;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type DbPoolState = Arc<DbPool>;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// #[utoipa::path(
//     get,
//     path = "/health",
//     responses(
//         (status = 200, description = "Service is healthy", body = HealthResponse),
//         (status = 503, description = "Service is unhealthy")
//     )
// )]
pub async fn health_check(
    State(pool): State<DbPoolState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    let timestamp = chrono::Utc::now();

    // Check database connectivity
    let database_status = match crate::models::health_check(&pool).await {
        Ok(_) => "healthy".to_string(),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            "unhealthy".to_string()
        }
    };

    let overall_status = if database_status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        database: database_status.clone(),
        timestamp,
    };

    if overall_status == "healthy" {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

// #[utoipa::path(
//     get,
//     path = "/ready",
//     responses(
//         (status = 200, description = "Service is ready"),
//         (status = 503, description = "Service is not ready")
//     )
// )]
pub async fn readiness_check(State(pool): State<DbPoolState>) -> Result<StatusCode, StatusCode> {
    match crate::models::health_check(&pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

// #[utoipa::path(
//     get,
//     path = "/live",
//     responses(
//         (status = 200, description = "Service is alive")
//     )
// )]
pub async fn liveness_check() -> StatusCode {
    StatusCode::OK
}

pub async fn get_api_health() -> Json<serde_json::Value> {
    use serde_json::json;
    Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "services": {
            "endpoint_service": "running",
            "swagger_service": "running",
            "database": "connected"
        }
    }))
}
