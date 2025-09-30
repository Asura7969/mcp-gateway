use axum::{body::Body, http::Request, middleware::Next, response::IntoResponse};
use tracing::{debug, error, info};

/// Middleware function to log all incoming requests
pub async fn log_requests(req: Request<Body>, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    // Log the incoming request
    info!("Incoming request: {} {}", method, uri);

    // Optionally log headers (be careful with sensitive data)
    for (name, value) in headers.iter() {
        // Skip logging sensitive headers like authorization
        if name != "authorization" && name != "cookie" {
            match value.to_str() {
                Ok(value_str) => {
                    debug!("Header: {}: {}", name, value_str);
                }
                Err(_) => {
                    debug!("Header: {}: (binary data)", name);
                }
            }
        }
    }

    // Process the request
    let response = next.run(req).await;

    // Log the response status
    let status = response.status();
    info!("Response status for {} {}: {}", method, uri, status);

    // Log error responses
    if status.is_client_error() || status.is_server_error() {
        error!("Error response for {} {}: {}", method, uri, status);
    }

    response
}
