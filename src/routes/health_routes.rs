use crate::handlers::get_api_health;
use crate::state::MergeState;
use axum::{routing::get, Router};

/// 创建健康检查路由
pub fn create_health_routes() -> Router<MergeState> {
    Router::new()
        // Health check routes
        .route("/health", get(get_api_health))
        .route("/ready", get(|| async { "Ready" }))
        .route("/live", get(|| async { "Live" }))
}
