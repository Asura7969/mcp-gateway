use crate::models::{DbPool, Endpoint, EndpointStatus};
use crate::services::mcp_service::{McpError, McpRequest, McpResponse};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

/// SSE Server Transport Handler
/// This implementation is similar to the Python SDK's SseServerTransport.handle_post_message
pub struct SseServerTransport {
    db_pool: DbPool,
    mcp_service: Arc<crate::services::mcp_service::McpService>,
}

impl SseServerTransport {
    pub fn new(
        db_pool: DbPool,
        mcp_service: Arc<crate::services::mcp_service::McpService>,
    ) -> Self {
        Self {
            db_pool,
            mcp_service,
        }
    }

    /// Handle POST message (similar to Python SDK's handle_post_message)
    /// This method receives incoming POST requests with client messages that link to a
    /// previously-established SSE session.
    pub async fn handle_post_message(
        &self,
        endpoint_id: Uuid,
        Query(params): Query<std::collections::HashMap<String, String>>,
        request_body: String,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        tracing::info!(
            "处理SSE POST消息 - 端点ID: {}, 请求体长度: {}",
            endpoint_id,
            request_body.len()
        );
        tracing::debug!("SSE POST消息内容: {}", request_body);

        // Extract session_id from query parameters (兼容Python SDK使用session_id)
        let session_id_str = params
            .get("session_id")
            .or_else(|| params.get("sessionId"))
            .ok_or_else(|| {
                tracing::error!("SSE POST消息失败 - 缺少session_id参数");
                (
                    StatusCode::BAD_REQUEST,
                    "Missing session_id parameter".to_string(),
                )
            })?;

        let session_id = Uuid::parse_str(session_id_str).map_err(|e| {
            tracing::error!(
                "SSE POST消息失败 - 无效的session_id: {} - 错误: {}",
                session_id_str,
                e
            );
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid session_id: {}", e),
            )
        })?;

        tracing::info!("SSE POST消息 - 解析session_id成功: {}", session_id);

        // Verify endpoint exists and is running
        let endpoint = self.get_endpoint(endpoint_id).await.map_err(|e| {
            tracing::error!(
                "SSE POST消息失败 - 端点不存在: {} - 错误: {}",
                endpoint_id,
                e
            );
            (StatusCode::NOT_FOUND, e.to_string())
        })?;

        if endpoint.status != EndpointStatus::Running {
            tracing::warn!(
                "SSE POST消息失败 - 端点状态不正确: {:?} (需要Running)",
                endpoint.status
            );
            return Err((StatusCode::CONFLICT, "Endpoint is not running".to_string()));
        }

        tracing::info!(
            "SSE POST消息 - 端点验证通过: {}, 状态: {:?}",
            endpoint.name,
            endpoint.status
        );

        // Check if session exists
        let tx = crate::handlers::mcp_transport_handler::get_sse_session(&session_id)
            .await
            .ok_or_else(|| {
                tracing::error!("SSE POST消息失败 - 未找到会话: {}", session_id);
                (StatusCode::NOT_FOUND, "Session not found".to_string())
            })?;

        tracing::debug!("SSE POST消息 - 会话验证通过: {}", session_id);

        // Parse MCP request
        let mcp_request: McpRequest = serde_json::from_str(&request_body).map_err(|e| {
            tracing::error!("SSE POST消息失败 - JSON解析错误: {}", e);
            (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e))
        })?;

        // Get the request ID before processing
        let request_id = mcp_request.id.clone();
        tracing::info!(
            "SSE POST消息 - MCP方法: {}, 请求ID: {:?}",
            mcp_request.method,
            request_id
        );

        // Process MCP request
        let response = self
            .process_mcp_request(&endpoint, mcp_request)
            .await
            .map_err(|e| {
                tracing::error!("SSE POST消息失败 - MCP处理错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

        tracing::debug!("SSE POST消息 - MCP处理完成, 请求ID: {:?}", request_id);

        // Convert response to JSON string
        let response_json = serde_json::to_value(&response).map_err(|e| {
            tracing::error!("SSE POST消息失败 - 响应序列化错误: {}", e);
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

        // Convert to JSON string for SSE
        let response_string = serde_json::to_string(&cleaned_response).map_err(|e| {
            tracing::error!("SSE POST消息失败 - 最终序列化错误: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize response: {}", e),
            )
        })?;

        tracing::info!(
            "SSE POST消息成功 - 响应长度: {}, 请求ID: {:?}",
            response_string.len(),
            request_id
        );
        tracing::debug!("SSE响应内容: {}", response_string);

        // Send the response through the SSE stream as a "message" event
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
                crate::handlers::mcp_transport_handler::remove_sse_session(&session_id).await;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to send response through SSE".to_string(),
                ));
            }
        }

        // Return 202 Accepted to indicate the message was accepted
        Ok(Response::builder()
            .status(StatusCode::ACCEPTED)
            .header("Content-Type", "text/plain")
            .body(Body::from("Accepted"))
            .unwrap())
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

    async fn process_mcp_request(
        &self,
        endpoint: &Endpoint,
        request: McpRequest,
    ) -> Result<McpResponse> {
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
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::Value::Null),
                    error: None,
                }
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

        Ok(response)
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

// Route handlers for SSE transport
// 注意：这些处理函数现在将重用现有的MCP传输处理程序中的SSE会话存储

/// SSE POST message handler (similar to Python SDK's handle_post_message)
/// This handler is compatible with the existing MCP transport implementation
pub async fn handle_sse_post_message(
    State(app_state): State<crate::state::AppState>,
    Path(endpoint_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    body: String,
) -> Result<Response<Body>, (StatusCode, String)> {
    // 重用现有的MCP传输处理程序来处理SSE POST消息
    let handler = crate::handlers::mcp_transport_handler::McpTransportHandler::new(
        (*app_state.endpoint_service.get_pool()).clone(),
        app_state.mcp_service.clone(),
    );
    handler
        .handle_sse_request(endpoint_id, Query(params), body)
        .await
}
