use crate::handlers::get_system_status;
use crate::state::MergeState;
use axum::{routing::get, Router};

/// 创建系统状态路由
pub fn create_system_routes() -> Router<MergeState> {
    Router::new()
        // System status route
        .route("/api/system/status", get(get_system_status))
}
