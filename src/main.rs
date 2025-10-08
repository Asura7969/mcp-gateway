mod config;
mod handlers;
mod middleware;
mod models;
mod services;
mod state;
mod utils;
mod error;
mod tests;

use axum::{
    routing::{get, post},
    Router,
};
use rmcp::transport::common::server_side_http::DEFAULT_AUTO_PING_INTERVAL;
use rmcp::transport::sse_server::{
    post_event_handler, sse_handler, App, ConnectionMsg, SseServerConfig,
};
use rmcp::transport::{SseServer, StreamableHttpServerConfig, StreamableHttpService};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::DB_POOL;
use crate::services::{McpService, SessionService};
use crate::utils::MonitoredSessionManager;
use config::Settings;
use handlers::*;
use middleware::{cors_layer, logging};
use models::create_pool;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use services::{EndpointService, SwaggerService, StartupLoaderService};
use state::AppState;
use tokio::sync::mpsc::UnboundedReceiver;
use tower::ServiceBuilder;
use utils::shutdown_signal;
use crate::middleware::stream_requests_interceptor;

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
    let external_pool = create_pool(
        &settings.database.url,
        settings.database.mcp_call_max_connections,
    )
    .await?;
    DB_POOL
        .set(external_pool)
        .expect("external_pool already initialized");

    let pool = create_pool(&settings.database.url, settings.database.max_connections).await?;
    tracing::info!("Database connection pool created");
    let db_pool = Arc::new(pool);

    // Create services
    let endpoint_service = Arc::new(EndpointService::new((*db_pool).clone()));
    let swagger_service = Arc::new(SwaggerService::new(EndpointService::new(
        (*db_pool).clone(),
    )));
    let mcp_service = Arc::new(McpService::new((*db_pool).clone()));
    
    // Create interface relation state
    let interface_relation_state = InterfaceRelationState::new().await
        .map_err(|e| anyhow::anyhow!("Failed to create interface relation state: {}", e))?;

    // 自动加载endpoints表中的swagger信息到SurrealDB
    tracing::info!("开始自动加载endpoints表中的swagger信息...");
    let startup_loader = StartupLoaderService::new(
        endpoint_service.clone(),
        interface_relation_state.service.clone(),
    );
    
    if let Err(e) = startup_loader.load_all_swagger_data().await {
        tracing::error!("自动加载swagger数据失败: {}", e);
        // 注意：这里不返回错误，允许应用继续启动，即使swagger数据加载失败
        tracing::warn!("应用将继续启动，但swagger数据可能不完整");
    } else {
        tracing::info!("swagger数据自动加载完成");
    }

    let addr = format!("{}:{}", settings.server.host, settings.server.port);

    let config = SseServerConfig {
        bind: addr.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    // 统计sse连接数
    let (connect_tx, connect_rx) = tokio::sync::mpsc::unbounded_channel();

    let (app, transport_rx) = App::new_v2(
        config.post_path.clone(),
        config.sse_keep_alive.unwrap_or(DEFAULT_AUTO_PING_INTERVAL),
        Some(connect_tx.clone()),
    );

    let app_state = AppState::new(
        endpoint_service,
        swagger_service,
        mcp_service.clone(),
        (*db_pool).clone(),
        connect_tx,
    );

    let session_service = Arc::new(SessionService::new((*db_pool).clone()));

    session_counter(connect_rx, session_service.clone());

    let sse_server = SseServer {
        transport_rx,
        config,
    };

    let session_manager =
        MonitoredSessionManager::new(LocalSessionManager::default(), session_service);

    let stream_http_service = StreamableHttpService::new(
        || Ok(Adapter::new()),
        session_manager.into(),
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
        // Connection tracking routes
        .route("/api/connections/endpoint", get(get_endpoint_connections))
        .route(
            "/api/connections/endpoint/count",
            get(get_endpoint_connection_count),
        )
        .route(
            "/api/connections/time-series",
            get(get_time_series_connection_counts),
        )
        // Interface relation routes
        .merge(create_interface_relation_routes().with_state(interface_relation_state))
        // Swagger UI for Interface Relations API
        // .route("/api-docs/openapi.json", get(|| async { axum::Json(InterfaceRelationApiDoc::openapi()) }))
        // rmcp handle
        // .nest("/{endpoint_id}", sse_router)
        .route("/{endpoint_id}/sse", get(sse_handler))
        .route("/message", post(post_event_handler))
        .nest_service("/stream", stream_http_service)
        .layer(
            ServiceBuilder::new()
                .layer(cors_layer())
                .layer(axum::middleware::from_fn(logging::log_requests))
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    stream_requests_interceptor)
                )
        )
        .with_state(merge_state);

    let ct = sse_server.config.ct.child_token();

    // Create server
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    // Create enhanced shutdown signal handler
    let shutdown_future = async move {
        shutdown_signal().await;
        ct.cancelled().await;
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

/// session连接计数器
fn session_counter(
    mut connect_rx: UnboundedReceiver<ConnectionMsg>,
    session_service: Arc<SessionService>,
) {
    tokio::task::spawn(async move {
        loop {
            match connect_rx.recv().await {
                Some(ConnectionMsg::Connect(endpoint_id, session_id, mcp_type)) => {
                    session_service
                        .add_session(endpoint_id, session_id, mcp_type)
                        .await;
                }
                Some(ConnectionMsg::Disconnect(endpoint_id, session_id, mcp_type)) => {
                    session_service
                        .remove_session(endpoint_id, session_id, mcp_type)
                        .await;
                }
                None => {}
            }
        }
    });
}
