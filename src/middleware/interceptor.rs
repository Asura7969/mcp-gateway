use crate::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use rmcp::transport::common::http_header::HEADER_SESSION_ID;
use rmcp::transport::sse_server::{ConnectionMsg, McpType};

pub async fn stream_requests_interceptor(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let uri = req.uri().clone();
    let method = req.method().clone();

    if matches!(method, Method::POST) && uri.path().starts_with("/stream/") {
        let headers = req.headers().clone();
        let session_id = headers.get(HEADER_SESSION_ID).and_then(|v| v.to_str().ok());
        if let Some(session_id) = session_id {
            // 截取endpoint_id
            let (_stream_prefix, endpoint_id) = uri.path().split_at(8);
            // 创建连接
            if let Err(e) = state.connect_tx.send(ConnectionMsg::Connect(
                endpoint_id.to_string(),
                session_id.to_string().into(),
                McpType::STREAMABLE,
            )) {
                tracing::warn!("Failed to send connection msg: {}", e);
            };
        }
    }

    next.run(req).await
}
