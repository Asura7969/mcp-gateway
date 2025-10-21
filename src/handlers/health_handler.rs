use crate::utils::get_china_time;
use axum::response::Json;

pub async fn get_api_health() -> Json<serde_json::Value> {
    use serde_json::json;
    Json(json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": get_china_time().to_rfc3339(),
        "version": "1.0.0",
        "services": {
            "endpoint_service": "running",
            "swagger_service": "running",
            "database": "connected"
        }
    }))
}

pub async fn actuator_health() -> Json<serde_json::Value> {
    use serde_json::json;
    Json(json!({
        "status": "up"
    }))
}
