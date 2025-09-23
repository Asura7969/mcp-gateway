use crate::services::{EndpointService, SwaggerService};
use crate::utils::ShutdownCoordinator;
use axum::extract::FromRef;
use rmcp::transport::sse_server::App;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub endpoint_service: Arc<EndpointService>,
    pub swagger_service: Arc<SwaggerService>,
    pub mcp_service: Arc<crate::services::mcp_service::McpService>,
    pub shutdown_coordinator: ShutdownCoordinator,
}

impl AppState {
    pub fn new(
        endpoint_service: Arc<EndpointService>,
        swagger_service: Arc<SwaggerService>,
        mcp_service: Arc<crate::services::mcp_service::McpService>,
        shutdown_coordinator: ShutdownCoordinator,
    ) -> Self {
        Self {
            endpoint_service,
            swagger_service,
            mcp_service,
            shutdown_coordinator,
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
