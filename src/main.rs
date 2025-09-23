mod config;
mod handlers;
mod middleware;
mod models;
mod services;
mod state;
mod utils;

use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rmcp::transport::common::server_side_http::DEFAULT_AUTO_PING_INTERVAL;
use rmcp::transport::sse_server::{post_event_handler, sse_handler, App, SseServerConfig};
use rmcp::transport::{SseServer, StreamableHttpServerConfig, StreamableHttpService};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::DB_POOL;
use crate::services::McpService;
use config::Settings;
use handlers::*;
use http_body_util::BodyExt;
use middleware::{cors_layer, track_connection};
use models::create_pool;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use services::{EndpointService, SwaggerService};
use state::AppState;
use utils::{graceful_shutdown_with_timeout, shutdown_signal, ShutdownCoordinator};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "mcp_gateway=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::new().unwrap_or_else(|_| {
        tracing::warn!("Failed to load configuration, using defaults");
        Settings::default()
    });

    tracing::info!("Starting MCP Gateway server...");
    tracing::info!("Configuration: {:?}", settings);

    // Create database connection pool
    let pool = create_pool(&settings.database.url, settings.database.max_connections).await?;
    tracing::info!("Database connection pool created");
    let db_pool = Arc::new(pool);

    let external_pool = create_pool(
        &settings.database.url,
        settings.database.mcp_call_max_connections,
    )
    .await?;
    DB_POOL
        .set(external_pool)
        .expect("external_pool already initialized");
    // Create services
    let endpoint_service = Arc::new(EndpointService::new((*db_pool).clone()));
    let swagger_service = Arc::new(SwaggerService::new(EndpointService::new(
        (*db_pool).clone(),
    )));
    let mcp_service = Arc::new(McpService::new((*db_pool).clone()));

    // Create shutdown coordinator
    let shutdown_coordinator = ShutdownCoordinator::new();

    let app_state = AppState::new(
        endpoint_service,
        swagger_service,
        mcp_service.clone(),
        shutdown_coordinator.clone(),
    );

    let addr = format!("{}:{}", settings.server.host, settings.server.port);

    let config = SseServerConfig {
        bind: addr.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    let (app, transport_rx) = App::new(
        config.post_path.clone(),
        config.sse_keep_alive.unwrap_or(DEFAULT_AUTO_PING_INTERVAL),
    );

    let sse_server = SseServer {
        transport_rx,
        config,
    };

    let stream_http_service = StreamableHttpService::new(
        || Ok(Adapter::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig {
            sse_keep_alive: Some(Duration::from_secs(60)),
            stateful_mode: true,
        },
    );

    // let (sse_server, router) = SseServer::new(config);
    let merge_state = state::MergeState {
        app_state: app_state.clone(),
        app,
    };

    // Build application router with API endpoints
    let app = Router::new()
        // Health check routes
        .route("/health", get(get_api_health))
        .route("/ready", get(|| async { "Ready" }))
        .route("/live", get(|| async { "Live" }))
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
        // Metrics routes
        .route("/api/metrics/endpoints", get(get_all_endpoint_metrics))
        // Swagger conversion route
        .route("/api/swagger", post(convert_swagger_to_mcp))
        // Health API route
        .route("/api/health", get(get_api_health))
        // System status route
        .route("/api/system/status", get(get_system_status))
        // rmcp handle
        // .nest("/{endpoint_id}", sse_router)
        .route("/{endpoint_id}/sse", get(sse_handler))
        .route("/message", post(post_event_handler))
        .nest_service("/stream", stream_http_service)
        // Add CORS middleware
        .layer(cors_layer())
        // .layer(axum::middleware::from_fn(print_request_response))
        // Add connection tracking middleware
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            track_connection,
        ))
        // Add shared state
        .with_state(merge_state);

    let ct = sse_server.config.ct.child_token();

    // Create server
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    if settings.monitoring.enabled {
        tracing::info!("Monitoring enabled");
    }

    // Create enhanced shutdown signal handler
    let shutdown_future = async move {
        shutdown_signal().await;
        ct.cancelled().await;
        // Check for force shutdown flag (could be from environment or command line args)
        let force_shutdown = std::env::var("FORCE_SHUTDOWN").is_ok();
        let timeout_duration = Duration::from_secs(
            std::env::var("SHUTDOWN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30), // Default 30 seconds
        );

        graceful_shutdown_with_timeout(shutdown_coordinator, timeout_duration, force_shutdown)
            .await;
    };

    // Start server with enhanced graceful shutdown
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_future);

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "sse server shutdown with error");
        }
    });
    let ct = sse_server.with_service(Adapter::new);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
