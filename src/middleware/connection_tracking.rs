use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use tracing::{info, error};
use uuid::Uuid;
use crate::models::DB_POOL;
use std::collections::HashMap;

/// Middleware function to track SSE and Streamable connections
pub async fn track_connections(req: Request<Body>, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let path = uri.path();
    
    // Check if this is a connection request we need to track
    let (endpoint_id, session_id, transport_type) = extract_connection_info(&method, path, &headers);
    
    // If we have connection info, handle it
    if let (Some(ep_id), Some(sess_id), Some(t_type)) = (&endpoint_id, &session_id, transport_type) {
        match method {
            // New connection
            axum::http::Method::POST | axum::http::Method::GET => {
                if let Err(e) = handle_new_connection(ep_id, sess_id, t_type).await {
                    error!("Failed to handle new connection: {}", e);
                }
            },
            // Connection closed
            axum::http::Method::DELETE => {
                if let Err(e) = handle_closed_connection(ep_id, sess_id).await {
                    error!("Failed to handle closed connection: {}", e);
                }
            },
            _ => {}
        }
    }
    
    // Process the request
    let response = next.run(req).await;
    
    response
}

/// Extract connection information from request
fn extract_connection_info(
    method: &axum::http::Method,
    path: &str,
    headers: &axum::http::HeaderMap,
) -> (Option<Uuid>, Option<Uuid>, Option<i16>) {
    // For Streamable connections: POST /stream/{endpoint_id} or DELETE /stream/{endpoint_id}
    if path.starts_with("/stream/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 3 {
            if let Ok(endpoint_id) = Uuid::parse_str(parts[2]) {
                // Try to get session_id from headers
                if let Some(session_header) = headers.get("mcp-session-id") {
                    if let Ok(session_str) = session_header.to_str() {
                        if let Ok(session_id) = Uuid::parse_str(session_str) {
                            return (Some(endpoint_id), Some(session_id), Some(2)); // 2 for streamable
                        }
                    }
                }
            }
        }
    }
    
    // For SSE connections: POST /message?sessionId={session_id}&endpointId={endpoint_id}
    if path.starts_with("/message") && method == axum::http::Method::POST {
        if let Some(query) = uri::query(path) {
            let params = parse_query_params(query);
            if let (Some(session_str), Some(endpoint_str)) = (params.get("sessionId"), params.get("endpointId")) {
                if let (Ok(session_id), Ok(endpoint_id)) = (Uuid::parse_str(session_str), Uuid::parse_str(endpoint_str)) {
                    return (Some(endpoint_id), Some(session_id), Some(1)); // 1 for sse
                }
            }
        }
    }
    
    (None, None, None)
}

/// Parse query parameters from a query string
fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            params.insert(key.to_string(), value.to_string());
        }
    }
    params
}

/// Handle a new connection
async fn handle_new_connection(endpoint_id: &Uuid, session_id: &Uuid, transport_type: i16) -> Result<(), anyhow::Error> {
    let pool = DB_POOL.get().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    
    // Insert or update the session log
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO endpoint_session_logs (id, endpoint_id, session_id, transport_type) VALUES (?, ?, ?, ?) 
         ON DUPLICATE KEY UPDATE disconnect_at = connect_at"
    )
    .bind(id.to_string())
    .bind(endpoint_id.to_string())
    .bind(session_id.to_string())
    .bind(transport_type)
    .execute(pool)
    .await?;
    
    // Increment the connection count
    let count_result = sqlx::query(
        "INSERT INTO endpoint_connection_counts (id, endpoint_id, connect_num) VALUES (?, ?, 1) 
         ON DUPLICATE KEY UPDATE connect_num = connect_num + 1"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(endpoint_id.to_string())
    .execute(pool)
    .await;
    
    match count_result {
        Ok(_) => {
            info!("Incremented connection count for endpoint {}", endpoint_id);
        },
        Err(e) => {
            error!("Failed to update connection count for endpoint {}: {}", endpoint_id, e);
        }
    }
    
    Ok(())
}

/// Handle a closed connection
async fn handle_closed_connection(endpoint_id: &Uuid, session_id: &Uuid) -> Result<(), anyhow::Error> {
    let pool = DB_POOL.get().ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;
    
    // Update the disconnect time
    sqlx::query(
        "UPDATE endpoint_session_logs SET disconnect_at = NOW() WHERE endpoint_id = ? AND session_id = ?"
    )
    .bind(endpoint_id.to_string())
    .bind(session_id.to_string())
    .execute(pool)
    .await?;
    
    // Decrement the connection count
    let count_result = sqlx::query(
        "UPDATE endpoint_connection_counts SET connect_num = GREATEST(0, connect_num - 1) WHERE endpoint_id = ?"
    )
    .bind(endpoint_id.to_string())
    .execute(pool)
    .await;
    
    match count_result {
        Ok(_) => {
            info!("Decremented connection count for endpoint {}", endpoint_id);
        },
        Err(e) => {
            error!("Failed to update connection count for endpoint {}: {}", endpoint_id, e);
        }
    }
    
    Ok(())
}

/// Helper module for URI parsing
mod uri {
    pub fn query(path_and_query: &str) -> Option<&str> {
        path_and_query.split('?').nth(1)
    }
}