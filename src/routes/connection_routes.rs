use crate::handlers::{
    get_endpoint_connection_count, get_endpoint_connections, get_time_series_connection_counts,
};
use crate::state::MergeState;
use axum::{routing::get, Router};

/// 创建连接跟踪路由
pub fn create_connection_routes() -> Router<MergeState> {
    Router::new()
        // Connection tracking routes
        .route("/api/connections/endpoint", get(get_endpoint_connections))
        .route(
            "/api/connections/endpoint/count",
            get(get_endpoint_connection_count),
        )
        .route(
            "/api/connections/time-series",
            get(get_time_series_connection_counts),
        )
}
