use crate::handlers::{create_endpoint, delete_endpoint, get_endpoint, get_endpoint_metrics, list_endpoints, list_endpoints_paginated, start_endpoint, stop_endpoint, sync_endpoint_vector, update_endpoint};
use crate::state::MergeState;
use axum::{
    routing::{get, post},
    Router,
};

/// 创建端点管理路由
pub fn create_endpoint_routes() -> Router<MergeState> {
    Router::new()
        // Endpoint management routes
        .route("/api/endpoint", post(create_endpoint).get(list_endpoints))
        .route("/api/endpoints", get(list_endpoints_paginated))
        .route(
            "/api/endpoint/{id}",
            get(get_endpoint)
                .put(update_endpoint)
                .delete(delete_endpoint),
        )
        .route("/api/endpoint/{id}/metrics", get(get_endpoint_metrics))
        .route("/api/endpoint/{id}/start", post(start_endpoint))
        .route("/api/endpoint/{id}/stop", post(stop_endpoint))
        .route("/api/endpoint/{name}/sync_vector", post(sync_endpoint_vector))
}
