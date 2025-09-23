use crate::models::{DbPool, Endpoint, EndpointStatus};
use crate::services::mcp_service::{McpError, McpRequest, McpResponse};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Response, Sse},
};
use futures::{stream::Stream, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;

/// MCP Transport types
#[derive(Debug, Clone)]
pub enum McpTransport {
    Stdio,
    Sse,
    Streamable,
}

// Session storage for SSE connections
lazy_static::lazy_static! {
    static ref SSE_SESSIONS: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Result<axum::response::sse::Event, std::convert::Infallible>>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

// Public functions to access SSE sessions
pub async fn get_sse_session(
    session_id: &Uuid,
) -> Option<mpsc::UnboundedSender<Result<axum::response::sse::Event, std::convert::Infallible>>> {
    let sessions = SSE_SESSIONS.read().await;
    sessions.get(session_id).cloned()
}

pub async fn remove_sse_session(session_id: &Uuid) {
    let mut sessions = SSE_SESSIONS.write().await;
    sessions.remove(session_id);
}

/// Multi-transport MCP server handler
pub struct McpTransportHandler {
    db_pool: DbPool,
    mcp_service: Arc<crate::services::mcp_service::McpService>,
}

impl McpTransportHandler {
    pub fn new(
        db_pool: DbPool,
        mcp_service: Arc<crate::services::mcp_service::McpService>,
    ) -> Self {
        Self {
            db_pool,
            mcp_service,
        }
    }

    /// Handle stdio MCP connection (for command-line clients)
    pub async fn handle_stdio(
        &self,
        endpoint_id: Uuid,
        request_body: String,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        tracing::info!(
            "Stdio请求处理开始 - 端点ID: {}, 请求体长度: {}",
            endpoint_id,
            request_body.len()
        );
        tracing::debug!("Stdio请求内容: {}", request_body);

        // Verify endpoint exists and is running
        let endpoint = self.get_endpoint(endpoint_id).await.map_err(|e| {
            tracing::error!("Stdio请求失败 - 端点不存在: {} - 错误: {}", endpoint_id, e);
            (StatusCode::NOT_FOUND, e.to_string())
        })?;

        if endpoint.status != EndpointStatus::Running {
            tracing::warn!(
                "Stdio请求失败 - 端点状态不正确: {:?} (需要Running)",
                endpoint.status
            );
            return Err((StatusCode::CONFLICT, "Endpoint is not running".to_string()));
        }

        tracing::info!(
            "Stdio请求 - 端点验证通过: {}, 状态: {:?}",
            endpoint.name,
            endpoint.status
        );

        // Parse MCP request from the request body
        let mcp_request: McpRequest = serde_json::from_str(&request_body).map_err(|e| {
            tracing::error!("Stdio请求失败 - JSON解析错误: {}", e);
            (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e))
        })?;

        tracing::info!(
            "Stdio请求 - 解析MCP请求成功, 方法: {}, 请求ID: {:?}",
            mcp_request.method,
            mcp_request.id
        );

        // Process MCP request
        let response = self
            .process_mcp_request(&endpoint, mcp_request)
            .await
            .map_err(|e| {
                tracing::error!("Stdio请求失败 - MCP处理错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

        tracing::debug!("Stdio请求 - MCP处理完成");

        // Convert to JSON and handle error field properly
        let mut response_json = serde_json::to_value(&response).map_err(|e| {
            tracing::error!("Stdio请求失败 - 响应序列化错误: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize response: {}", e),
            )
        })?;

        if let Some(obj) = response_json.as_object_mut() {
            // Remove error field if it's null
            if let Some(error_value) = obj.get("error") {
                if error_value.is_null() {
                    obj.remove("error");
                }
            }
        }

        let response_string = serde_json::to_string(&response_json).map_err(|e| {
            tracing::error!("Stdio请求失败 - 最终序列化错误: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize response: {}", e),
            )
        })?;

        tracing::info!("Stdio请求成功 - 响应长度: {}", response_string.len());
        tracing::debug!("Stdio响应内容: {}", response_string);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(response_string))
            .unwrap())
    }

    async fn process_mcp_request(
        &self,
        endpoint: &Endpoint,
        request: McpRequest,
    ) -> Result<String> {
        let response = match request.method.as_str() {
            "initialize" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({
                    "protocolVersion": "2025-03-26",
                    "capabilities": {
                        "tools": {
                            "listChanged": true
                        },
                        "resources": {
                            "subscribe": true
                        },
                        "prompts": {
                            "listChanged": true
                        },
                        "logging": {
                            "setLevel": true
                        }
                    },
                    "serverInfo": {
                        "name": format!("mcp-gateway-{}", endpoint.name),
                        "version": "1.0.0"
                    }
                })),
                error: None,
            },
            "notifications/initialized" => {
                // Handle initialized notification - no response needed for notifications
                return Ok(String::new());
            }
            "tools/list" => {
                // Parse swagger content and generate tools
                let swagger_spec: crate::models::SwaggerSpec =
                    serde_json::from_str(&endpoint.swagger_content)?;
                let tools = Self::generate_tools_from_swagger(&swagger_spec)?;

                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "tools": tools
                    })),
                    error: None,
                }
            }
            "resources/list" => {
                // For now, return empty resources list
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "resources": []
                    })),
                    error: None,
                }
            }
            "prompts/list" => {
                // For now, return empty prompts list
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "prompts": []
                    })),
                    error: None,
                }
            }
            "tools/call" => {
                let params = request
                    .params
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing parameters"))?;
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing tool name"))?;
                let empty_args = serde_json::json!({});
                let arguments = params.get("arguments").unwrap_or(&empty_args);

                let result = self
                    .mcp_service
                    .execute_tool_call(endpoint, tool_name, arguments)
                    .await?;

                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    })),
                    error: None,
                }
            }
            "ping" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({})),
                error: None,
            },
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        };

        // Convert to JSON and handle error field properly
        let mut response_json = serde_json::to_value(&response)?;
        if let Some(obj) = response_json.as_object_mut() {
            // Remove error field if it's null
            if let Some(error_value) = obj.get("error") {
                if error_value.is_null() {
                    obj.remove("error");
                }
            }
        }

        Ok(serde_json::to_string(&response_json)?)
    }

    /// Handle SSE transport connection
    pub async fn handle_sse(
        &self,
        endpoint_id: Uuid,
    ) -> Result<
        Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>,
        (StatusCode, String),
    > {
        tracing::info!("SSE连接请求 - 端点ID: {}", endpoint_id);

        // Verify endpoint exists and is running
        let endpoint = self.get_endpoint(endpoint_id).await.map_err(|e| {
            tracing::error!("SSE连接失败 - 端点不存在: {} - 错误: {}", endpoint_id, e);
            (StatusCode::NOT_FOUND, e.to_string())
        })?;

        tracing::info!(
            "SSE连接 - 端点信息: 名称={}, 状态={:?}",
            endpoint.name,
            endpoint.status
        );

        if endpoint.status != EndpointStatus::Running {
            tracing::warn!(
                "SSE连接失败 - 端点状态不正确: {:?} (需要Running)",
                endpoint.status
            );
            return Err((StatusCode::CONFLICT, "Endpoint is not running".to_string()));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let mcp_service = self.mcp_service.clone();

        tracing::info!("SSE连接 - 创建消息通道成功");

        // Create enhanced SSE stream that can handle MCP requests
        let stream = McpSseStream::new(rx, mcp_service.clone(), endpoint_id, endpoint.clone());
        tracing::debug!("SSE连接 - 创建流对象成功");

        // For SSE, we don't automatically send capabilities or tools
        // The client should send initialize and tools/list requests explicitly
        // This matches the official Inspector behavior

        // Just send a connection established event
        let session_id = uuid::Uuid::new_v4();

        // Store the session
        {
            let mut sessions = SSE_SESSIONS.write().await;
            sessions.insert(session_id, tx.clone());
        }

        tracing::info!("SSE连接 - 发送连接建立事件, 会话ID: {}", session_id);

        let event_data = format!("/message?sessionId={}", session_id);
        let sse_event = axum::response::sse::Event::default()
            .event("endpoint")
            .data(event_data.clone());

        match tx.send(Ok(sse_event)) {
            Ok(_) => {
                tracing::debug!("SSE连接 - 成功发送endpoint事件: {}", event_data);
            }
            Err(e) => {
                tracing::error!("SSE连接 - 发送endpoint事件失败: {}", e);
                // Remove the session if we can't send the event
                let mut sessions = SSE_SESSIONS.write().await;
                sessions.remove(&session_id);
            }
        }

        // Don't automatically generate tools - wait for explicit requests
        /*
        match serde_json::from_str::<crate::models::SwaggerSpec>(&endpoint.swagger_content) {
            Ok(swagger_spec) => {
                match Self::generate_tools_from_swagger(&swagger_spec) {
                    Ok(tools) => {
                        let tools_response = McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Some(serde_json::Value::String("tools_list".to_string())),
                            result: Some(serde_json::json!({
                                "tools": tools
                            })),
                            error: None,
                        };

                        // Convert to JSON and remove null error field
                        let mut tools_json = serde_json::to_value(&tools_response).unwrap();
                        if let Some(obj) = tools_json.as_object_mut() {
                            if obj.get("error").and_then(|v| v.as_null()).is_some() {
                                obj.remove("error");
                            }
                        }

                        let _ = tx.send(Ok(axum::response::sse::Event::default()
                            .event("message")
                            .data(serde_json::to_string(&tools_json).unwrap())));
                    }
                    Err(e) => {
                        tracing::error!("Failed to generate tools for SSE: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to parse swagger spec for SSE: {}", e);
            }
        }
        */

        let sse_response = Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(300)) // 5分钟保活间隔
                .text(""), // 空消息，减少带宽占用
        );

        tracing::debug!(
            "SSE连接建立成功 - 端点ID: {}, 端点名称: {}",
            endpoint_id,
            endpoint.name
        );

        Ok(sse_response)
    }

    /// Handle SSE MCP request (for bidirectional communication)
    /// Updated to be fully compatible with the official MCP protocol
    pub async fn handle_sse_request(
        &self,
        endpoint_id: Uuid,
        Query(params): Query<std::collections::HashMap<String, String>>,
        request_body: String,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        tracing::info!(
            "SSE请求处理开始 - 端点ID: {}, 请求体长度: {}",
            endpoint_id,
            request_body.len()
        );
        tracing::debug!("SSE请求内容: {}", request_body);

        // Extract session_id from query parameters (兼容Python SDK使用session_id)
        let session_id_str = params
            .get("session_id")
            .or_else(|| params.get("sessionId"))
            .ok_or_else(|| {
                tracing::error!("SSE请求失败 - 缺少session_id参数");
                (
                    StatusCode::BAD_REQUEST,
                    "Missing session_id parameter".to_string(),
                )
            })?;

        let session_id = Uuid::parse_str(session_id_str).map_err(|e| {
            tracing::error!(
                "SSE请求失败 - 无效的session_id: {} - 错误: {}",
                session_id_str,
                e
            );
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid session_id: {}", e),
            )
        })?;

        tracing::info!("SSE请求 - 解析session_id成功: {}", session_id);

        // Verify endpoint exists and is running
        let endpoint = self.get_endpoint(endpoint_id).await.map_err(|e| {
            tracing::error!("SSE请求失败 - 端点不存在: {} - 错误: {}", endpoint_id, e);
            (StatusCode::NOT_FOUND, e.to_string())
        })?;

        if endpoint.status != EndpointStatus::Running {
            tracing::warn!(
                "SSE请求失败 - 端点状态不正确: {:?} (需要Running)",
                endpoint.status
            );
            return Err((StatusCode::CONFLICT, "Endpoint is not running".to_string()));
        }

        tracing::info!(
            "SSE请求 - 端点验证通过: {}, 状态: {:?}",
            endpoint.name,
            endpoint.status
        );

        // Check if session exists
        let sessions = SSE_SESSIONS.read().await;
        let tx = sessions.get(&session_id).ok_or_else(|| {
            tracing::error!("SSE请求失败 - 未找到会话: {}", session_id);
            (StatusCode::NOT_FOUND, "Session not found".to_string())
        })?;
        let tx = tx.clone(); // Clone the sender for use
        drop(sessions); // Release the read lock

        tracing::debug!("SSE请求 - 会话验证通过: {}", session_id);

        // Parse MCP request
        let mcp_request: McpRequest = serde_json::from_str(&request_body).map_err(|e| {
            tracing::error!("SSE请求失败 - JSON解析错误: {}", e);
            (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e))
        })?;

        // Get the request ID before processing
        let request_id = mcp_request.id.clone();
        tracing::info!(
            "SSE请求 - MCP方法: {}, 请求ID: {:?}",
            mcp_request.method,
            request_id
        );

        // Process MCP request
        let response = self
            .process_mcp_request(&endpoint, mcp_request)
            .await
            .map_err(|e| {
                tracing::error!("SSE请求失败 - MCP处理错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

        tracing::debug!("SSE请求 - MCP处理完成, 请求ID: {:?}", request_id);

        // Convert response to JSON string
        let response_json = serde_json::to_value(&response).map_err(|e| {
            tracing::error!("SSE请求失败 - 响应序列化错误: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize response: {}", e),
            )
        })?;

        let mut cleaned_response = response_json.clone();
        if let Some(obj) = cleaned_response.as_object_mut() {
            // Remove error field if it's null
            if let Some(error_value) = obj.get("error") {
                if error_value.is_null() {
                    obj.remove("error");
                }
            }
        }

        // Return the response in SSE format (compatible with official MCP protocol)
        let response_string = serde_json::to_string(&cleaned_response).map_err(|e| {
            tracing::error!("SSE请求失败 - 最终序列化错误: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize response: {}", e),
            )
        })?;

        // Format as SSE event - Fully compatible with official MCP protocol
        let sse_response = format!("data: {}\n\n", response_string);

        tracing::info!(
            "SSE请求成功 - 响应长度: {}, 请求ID: {:?}",
            sse_response.len(),
            request_id
        );
        tracing::debug!("SSE响应内容: {}", sse_response);

        // Send the response through the SSE stream
        match tx.send(Ok(axum::response::sse::Event::default()
            .event("message")
            .data(response_string.clone())))
        {
            Ok(_) => {
                tracing::debug!("SSE响应发送成功");
            }
            Err(e) => {
                tracing::error!("SSE响应发送失败: {}", e);
                // Remove the session if we can't send the event
                let mut sessions = SSE_SESSIONS.write().await;
                sessions.remove(&session_id);
            }
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from("Accepted"))
            .unwrap())
    }

    /// Handle streamable MCP connection (for streaming responses)
    /// Fully compliant with the official Streamable HTTP transport specification
    pub async fn handle_streamable(
        &self,
        endpoint_id: Uuid,
        request_body: String,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        tracing::info!(
            "Streamable请求处理开始 - 端点ID: {}, 请求体长度: {}",
            endpoint_id,
            request_body.len()
        );
        tracing::debug!("Streamable请求内容: {}", request_body);

        // Verify endpoint exists and is running
        let endpoint = self.get_endpoint(endpoint_id).await.map_err(|e| {
            tracing::error!(
                "Streamable请求失败 - 端点不存在: {} - 错误: {}",
                endpoint_id,
                e
            );
            (StatusCode::NOT_FOUND, e.to_string())
        })?;

        if endpoint.status != EndpointStatus::Running {
            tracing::warn!(
                "Streamable请求失败 - 端点状态不正确: {:?} (需要Running)",
                endpoint.status
            );
            return Err((StatusCode::CONFLICT, "Endpoint is not running".to_string()));
        }

        tracing::info!(
            "Streamable请求 - 端点验证通过: {}, 状态: {:?}",
            endpoint.name,
            endpoint.status
        );

        // Parse MCP request
        let mcp_requests: Vec<McpRequest> =
            match serde_json::from_str::<Vec<McpRequest>>(&request_body) {
                Ok(requests) => {
                    tracing::debug!("Streamable请求 - 解析为多个请求，数量: {}", requests.len());
                    requests
                }
                Err(_) => {
                    // Try to parse as single request
                    match serde_json::from_str::<McpRequest>(&request_body) {
                        Ok(request) => {
                            tracing::debug!("Streamable请求 - 解析为单个请求");
                            vec![request]
                        }
                        Err(e) => {
                            tracing::error!("Streamable请求失败 - JSON解析错误: {}", e);
                            return Err((StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)));
                        }
                    }
                }
            };

        // Log the parsed requests
        for (i, req) in mcp_requests.iter().enumerate() {
            tracing::debug!(
                "Streamable请求[{}] - 方法: {}, 请求ID: {:?}",
                i,
                req.method,
                req.id
            );
        }

        // Check if all requests are responses or notifications (no need to send back response)
        let all_responses_or_notifications = mcp_requests.iter().all(|req| {
            req.id.is_none()
                || req.method == "cancelled"
                || req.method == "progress"
                || req.method.starts_with("notifications/")
        });

        if all_responses_or_notifications {
            tracing::info!("Streamable请求 - 所有请求都是响应或通知，返回202 Accepted");
            // For responses/notifications only, return 202 Accepted with no body
            return Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap());
        }

        // Check if any requests are present (need to send back response)
        let has_requests = mcp_requests.iter().any(|req| req.id.is_some());
        tracing::debug!("Streamable请求 - 是否包含需要响应的请求: {}", has_requests);

        if has_requests {
            // Create streaming response for requests that need responses
            let (tx, rx) = mpsc::unbounded_channel::<Result<String, std::io::Error>>();
            let mcp_service = self.mcp_service.clone();

            // Process each request and send responses
            for (i, mcp_request) in mcp_requests.into_iter().enumerate() {
                if mcp_request.id.is_some() {
                    let tx_clone = tx.clone();
                    let mcp_service_clone = mcp_service.clone();
                    let endpoint_clone = endpoint.clone();
                    let request_id = mcp_request.id.clone(); // 克隆request ID以避免移动问题

                    tracing::debug!("Streamable请求[{}] - 启动异步处理任务", i);

                    tokio::spawn(async move {
                        tracing::debug!("Streamable请求处理任务开始 - 请求ID: {:?}", request_id);

                        match Self::process_streamable_request(
                            mcp_service_clone,
                            endpoint_clone,
                            mcp_request,
                            tx_clone.clone(),
                        )
                        .await
                        {
                            Ok(_) => {
                                tracing::debug!(
                                    "Streamable请求处理任务完成 - 请求ID: {:?}",
                                    request_id
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Streamable请求处理任务错误 - 请求ID: {:?}, 错误: {}",
                                    request_id,
                                    e
                                );
                                let error_response = McpResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: request_id, // Use the cloned request ID for error responses
                                    result: None,
                                    error: Some(McpError {
                                        code: -32603,
                                        message: e.to_string(),
                                        data: None,
                                    }),
                                };
                                let error_json = match serde_json::to_string(&error_response) {
                                    Ok(json) => json,
                                    Err(e) => {
                                        tracing::error!("Streamable请求错误响应序列化失败: {}", e);
                                        return;
                                    }
                                };
                                let _ = tx_clone.send(Ok(format!("{}\n", error_json)));
                            }
                        }
                    });
                } else {
                    tracing::debug!("Streamable请求[{}] - 跳过无需响应的请求", i);
                }
            }

            // Create streaming body
            let stream = tokio_util::io::ReaderStream::new(tokio_util::io::StreamReader::new(
                UnboundedReceiverStream::new(rx).map(|item| item.map(|s| bytes::Bytes::from(s))),
            ));

            tracing::info!("Streamable请求处理完成 - 返回流式响应");

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .body(Body::from_stream(stream))
                .unwrap())
        } else {
            // No requests that need responses, return 202 Accepted
            tracing::info!("Streamable请求 - 没有需要响应的请求，返回202 Accepted");
            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap())
        }
    }

    /// Process a single streamable request
    async fn process_streamable_request(
        mcp_service: Arc<crate::services::mcp_service::McpService>,
        endpoint: Endpoint,
        request: McpRequest,
        tx: mpsc::UnboundedSender<Result<String, std::io::Error>>,
    ) -> Result<()> {
        tracing::debug!(
            "处理Streamable请求 - 方法: {}, 请求ID: {:?}",
            request.method,
            request.id
        );

        // Process the MCP request and send streaming responses
        let response = match request.method.as_str() {
            "initialize" => {
                tracing::info!("处理initialize请求 - 请求ID: {:?}", request.id);
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "protocolVersion": "2025-03-26",
                        "capabilities": {
                            "tools": {
                                "listChanged": true
                            },
                            "resources": {
                                "subscribe": true
                            },
                            "prompts": {
                                "listChanged": true
                            },
                            "logging": {
                                "setLevel": true
                            }
                        },
                        "serverInfo": {
                            "name": format!("mcp-gateway-{}", endpoint.name),
                            "version": "1.0.0"
                        }
                    })),
                    error: None,
                }
            }
            "notifications/initialized" => {
                tracing::info!(
                    "处理notifications/initialized请求 - 请求ID: {:?}",
                    request.id
                );
                // Handle initialized notification - no response needed for notifications
                let _ = tx.send(Ok(String::new()));
                return Ok(());
            }
            "tools/list" => {
                tracing::info!("处理tools/list请求 - 请求ID: {:?}", request.id);
                // Parse swagger content and generate tools
                let swagger_spec: crate::models::SwaggerSpec =
                    serde_json::from_str(&endpoint.swagger_content)?;
                let tools = Self::generate_tools_from_swagger(&swagger_spec)?;

                tracing::debug!("tools/list请求 - 生成工具数量: {}", tools.len());

                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "tools": tools
                    })),
                    error: None,
                }
            }
            "resources/list" => {
                tracing::info!("处理resources/list请求 - 请求ID: {:?}", request.id);
                // For now, return empty resources list
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "resources": []
                    })),
                    error: None,
                }
            }
            "prompts/list" => {
                tracing::info!("处理prompts/list请求 - 请求ID: {:?}", request.id);
                // For now, return empty prompts list
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "prompts": []
                    })),
                    error: None,
                }
            }
            "tools/call" => {
                tracing::info!("处理tools/call请求 - 请求ID: {:?}", request.id);
                // For tool calls, we can stream the execution progress
                let params = request
                    .params
                    .as_ref()
                    .ok_or_else(|| anyhow!("Missing parameters"))?;
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing tool name"))?;

                tracing::info!("执行工具调用: {}", tool_name);
                tracing::debug!("工具调用参数: {:?}", params);

                // Send progress update
                let progress = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: Some(serde_json::json!({
                        "type": "progress",
                        "message": format!("Executing tool: {}", tool_name)
                    })),
                    error: None,
                };
                let progress_json = serde_json::to_string(&progress)?;
                let _ = tx.send(Ok(format!("{}\n", progress_json)));

                // Execute the tool call
                let empty_args = serde_json::json!({});
                let arguments = params.get("arguments").unwrap_or(&empty_args);

                tracing::debug!("工具参数: {}", arguments);

                let result = mcp_service
                    .execute_tool_call(&endpoint, tool_name, arguments)
                    .await?;

                tracing::info!("工具调用执行完成: {}", tool_name);
                tracing::debug!("工具调用结果: {}", result);

                // Send final result
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    })),
                    error: None,
                }
            }
            method => {
                tracing::warn!("未知方法: {} - 请求ID: {:?}", method, request.id);
                // Return method not found error
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                }
            }
        };

        // Convert to JSON and handle error field properly
        let mut response_json = serde_json::to_value(&response)?;
        if let Some(obj) = response_json.as_object_mut() {
            // Remove error field if it's null
            if let Some(error_value) = obj.get("error") {
                if error_value.is_null() {
                    obj.remove("error");
                }
            }
        }

        let response_string = serde_json::to_string(&response_json)?;
        tracing::debug!("Streamable响应 - 内容长度: {}", response_string.len());

        let _ = tx.send(Ok(format!("{}\n", response_string)));

        Ok(())
    }

    async fn get_endpoint(&self, endpoint_id: Uuid) -> Result<Endpoint> {
        let endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE id = ?"
        )
        .bind(endpoint_id.to_string())
        .fetch_one(&self.db_pool)
        .await?;

        Ok(endpoint)
    }

    async fn create_stdio_script(&self, endpoint_id: Uuid, endpoint: &Endpoint) -> Result<String> {
        // Create a shell script that can be used as stdio transport
        let script = format!(
            r#"#!/bin/bash
# MCP stdio transport script for endpoint: {}
# Usage: ./mcp_stdio_{}.sh

ENDPOINT_ID="{}"
API_BASE="http://localhost:3000"

# Function to handle MCP stdio communication
mcp_stdio() {{
    while IFS= read -r line; do
        if [ ! -z "$line" ]; then
            response=$(curl -s -X POST "$API_BASE/mcp/$ENDPOINT_ID/streamable" \
                -H "Content-Type: application/json" \
                -d "$line")
            echo "$response"
        fi
    done
}}

# Start stdio communication
mcp_stdio
"#,
            endpoint.name, endpoint_id, endpoint_id
        );

        Ok(script)
    }

    fn generate_tools_from_swagger(
        swagger_spec: &crate::models::SwaggerSpec,
    ) -> Result<Vec<Value>> {
        let mut tools = Vec::new();

        for (path, path_item) in &swagger_spec.paths {
            let methods = [
                ("GET", &path_item.get),
                ("POST", &path_item.post),
                ("PUT", &path_item.put),
                ("DELETE", &path_item.delete),
                ("PATCH", &path_item.patch),
            ];

            for (method, operation_opt) in methods {
                if let Some(operation) = operation_opt {
                    // Use consistent naming without random UUID
                    let tool_name = operation.operation_id.clone().unwrap_or_else(|| {
                        format!(
                            "{}_{}_api",
                            method.to_lowercase(),
                            path.replace('/', "_")
                                .replace('{', "")
                                .replace('}', "")
                                .trim_start_matches('_')
                        )
                    });

                    let description = operation
                        .summary
                        .clone()
                        .or_else(|| operation.description.clone())
                        .unwrap_or_else(|| format!("{} {}", method, path));

                    // Build input schema with parameters from swagger spec
                    let mut properties = serde_json::Map::new();
                    let mut required = Vec::new();

                    // Add path parameters
                    if let Some(parameters) = &operation.parameters {
                        for param in parameters {
                            // 获取参数类型，如果schema存在则从schema中获取，否则默认为string
                            let param_type = if let Some(schema) = &param.schema {
                                schema.schema_type.clone().unwrap_or("string".to_string())
                            } else {
                                "string".to_string()
                            };

                            let property = serde_json::json!({
                                "type": param_type,
                                "description": param.description.clone().unwrap_or_default()
                            });

                            properties.insert(param.name.clone(), property);

                            // 检查参数是否为必需
                            if param.required.unwrap_or(false) {
                                required.push(param.name.clone());
                            }
                        }
                    }

                    // Add request body parameters for POST, PUT, PATCH
                    if ["POST", "PUT", "PATCH"].contains(&method) {
                        if let Some(request_body) = &operation.request_body {
                            // content字段不是Option类型，直接引用
                            let content = &request_body.content;
                            // Get the first media type
                            if let Some((_, media_type)) = content.iter().next() {
                                if let Some(schema) = &media_type.schema {
                                    // If schema has properties, add them to the tool input schema
                                    if let Some(schema_props) = &schema.properties {
                                        for (prop_name, prop_schema) in schema_props {
                                            let prop_type = prop_schema
                                                .schema_type
                                                .clone()
                                                .unwrap_or("string".to_string());
                                            let property = serde_json::json!({
                                                "type": prop_type,
                                                "description": format!("Property {}", prop_name)
                                            });
                                            properties.insert(prop_name.clone(), property);
                                        }
                                    }

                                    // Handle required fields
                                    if let Some(schema_required) = &schema.required {
                                        for req_item in schema_required {
                                            required.push(req_item.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let input_schema = serde_json::json!({
                        "type": "object",
                        "properties": properties,
                        "required": required
                    });

                    tools.push(serde_json::json!({
                        "name": tool_name,
                        "description": description,
                        "inputSchema": input_schema
                    }));
                }
            }
        }

        Ok(tools)
    }
}

/// SSE Stream implementation for MCP
pub struct McpSseStream {
    rx: mpsc::UnboundedReceiver<Result<axum::response::sse::Event, std::convert::Infallible>>,
    _mcp_service: Arc<crate::services::mcp_service::McpService>,
    _endpoint_id: Uuid,
    _endpoint: Endpoint,
}

impl McpSseStream {
    pub fn new(
        rx: mpsc::UnboundedReceiver<Result<axum::response::sse::Event, std::convert::Infallible>>,
        mcp_service: Arc<crate::services::mcp_service::McpService>,
        endpoint_id: Uuid,
        endpoint: Endpoint,
    ) -> Self {
        Self {
            rx,
            _mcp_service: mcp_service,
            _endpoint_id: endpoint_id,
            _endpoint: endpoint,
        }
    }
}

impl Stream for McpSseStream {
    type Item = Result<axum::response::sse::Event, std::convert::Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

// Route handlers for different transport protocols

/// Stdio transport handler
pub async fn handle_mcp_stdio_transport(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
    body: String,
) -> Result<Response<Body>, (StatusCode, String)> {
    let handler = McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler.handle_stdio(endpoint_id, body).await
}

/// SSE transport handler (GET for stream, POST for MCP requests)
/// Updated to support Streamable HTTP transport
pub async fn handle_mcp_sse_transport(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
) -> Result<
    Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>,
    (StatusCode, String),
> {
    let handler = McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler.handle_sse(endpoint_id).await
}

/// SSE MCP request handler (POST for sending MCP requests to SSE stream)
/// Updated to support Streamable HTTP transport
pub async fn handle_mcp_sse_request(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    body: String,
) -> Result<Response<Body>, (StatusCode, String)> {
    let handler = McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler
        .handle_sse_request(endpoint_id, Query(params), body)
        .await
}

/// Streamable transport handler
/// Updated to support Streamable HTTP transport specification
pub async fn handle_mcp_streamable_transport(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
    body: String,
) -> Result<Response<Body>, (StatusCode, String)> {
    let handler = McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler.handle_streamable(endpoint_id, body).await
}

/// Streamable HTTP GET handler for optional SSE stream
/// Added to fully support Streamable HTTP transport specification
pub async fn handle_mcp_streamable_get(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
) -> Result<
    Sse<impl Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>,
    (StatusCode, String),
> {
    let handler = McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler.handle_sse(endpoint_id).await
}
