use crate::handlers::get_all_endpoint_metrics;
use crate::state::MergeState;
use axum::{routing::get, Router};

/// 创建指标路由
pub fn create_metrics_routes() -> Router<MergeState> {
    Router::new()
        // Metrics routes
        .route("/api/metrics/endpoints", get(get_all_endpoint_metrics))
}
