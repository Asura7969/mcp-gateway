use crate::models::DbPool;
use crate::services::{EmbeddingService, EndpointService, SwaggerService};
use axum::extract::FromRef;
use rmcp::transport::sse_server::{App, ConnectionMsg};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub endpoint_service: Arc<EndpointService>,
    pub swagger_service: Arc<SwaggerService>,
    pub mcp_service: Arc<crate::services::mcp_service::McpService>,
    pub embedding_service: Arc<EmbeddingService>,
    pub pool: DbPool,
    pub connect_tx: tokio::sync::mpsc::UnboundedSender<ConnectionMsg>,
}

impl AppState {
    pub fn new(
        endpoint_service: Arc<EndpointService>,
        swagger_service: Arc<SwaggerService>,
        mcp_service: Arc<crate::services::mcp_service::McpService>,
        embedding_service: Arc<EmbeddingService>,
        pool: DbPool,
        connect_tx: tokio::sync::mpsc::UnboundedSender<ConnectionMsg>,
    ) -> Self {
        Self {
            endpoint_service,
            swagger_service,
            mcp_service,
            embedding_service,
            pool,
            connect_tx,
        }
    }
}

#[derive(Clone)]
pub struct MergeState {
    pub app_state: AppState,
    pub app: App,
}

impl FromRef<MergeState> for AppState {
    fn from_ref(merge_state: &MergeState) -> Self {
        merge_state.app_state.clone()
    }
}

impl FromRef<MergeState> for App {
    fn from_ref(app_version: &MergeState) -> Self {
        app_version.app.clone()
    }
}
