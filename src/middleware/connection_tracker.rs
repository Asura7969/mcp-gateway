use axum::{body::Body, extract::State, http::Request, middleware::Next, response::IntoResponse};

use crate::state::AppState;

/// Middleware function to track connections
pub async fn track_connection(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    // Check if shutdown is in progress
    if state.shutdown_coordinator.is_shutdown_in_progress() {
        return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    // Increment connection count
    state.shutdown_coordinator.increment_connections();

    // Process request
    let response = next.run(req).await;

    // Decrement connection count when done
    state.shutdown_coordinator.decrement_connections();

    response
}
