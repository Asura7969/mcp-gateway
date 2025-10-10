use crate::handlers::convert_swagger_to_mcp;
use crate::state::MergeState;
use axum::{routing::post, Router};

/// 创建Swagger转换路由
pub fn create_swagger_routes() -> Router<MergeState> {
    Router::new()
        // Swagger conversion route
        .route("/api/swagger", post(convert_swagger_to_mcp))
}
