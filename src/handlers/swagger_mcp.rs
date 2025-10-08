#![allow(dead_code)]

use crate::models::{Endpoint, DB_POOL};
use crate::utils::{
    build_base_url, build_url, extract_endpoint_id, extract_request_parts, parse_tool_name,
    update_metrics,
};
use anyhow::{anyhow, Error};
use reqwest::Client;
use rmcp::model::CallToolResult;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer, ServerHandler};
use serde_json::{json, Value};
use std::future::Future;
use uuid::Uuid;

#[derive(Clone)]
pub struct Adapter {
    http_client: Client,
}

impl Adapter {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    async fn inner_list_tools(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        tracing::info!("listing tools");

        let endpoint_id = if let Some(id) = self.get_endpoint_id(&context) {
            Ok(id)
        } else {
            Err(McpError::parse_error("not found endpoint", None))
        }?;
        if let Ok(endpoint) = self.get_endpoint(endpoint_id).await {
            let tools = <Vec<Tool>>::from(&endpoint);
            tracing::info!("tools size: {}", tools.len());
            Ok(ListToolsResult::with_all_items(tools))
        } else {
            tracing::info!("empty tools");
            Ok(ListToolsResult::with_all_items(vec![]))
        }
    }

    fn get_endpoint_id(&self, context: &RequestContext<RoleServer>) -> Option<Uuid> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            // let initialize_headers = &http_request_part.headers;
            let uri = &http_request_part.uri;
            if let Some(endpoint_id) = extract_endpoint_id(uri.to_string().as_str()) {
                tracing::info!("get endpoint id: {}", endpoint_id);
                return Some(Uuid::parse_str(endpoint_id.as_str()).unwrap());
            }
        }
        None
    }

    async fn inner_call_tool(
        &self,
        CallToolRequestParam { name, arguments }: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let endpoint_id = if let Some(id) = self.get_endpoint_id(&context) {
            Ok(id)
        } else {
            Err(McpError::parse_error("not found endpoint", None))
        }?;

        let arguments = arguments.map(|v| Value::Object(v)).unwrap_or(Value::Null);
        tracing::info!("call tool arguments: {}", arguments);
        match self
            .execute_tool_call_from_id(endpoint_id, name.as_ref(), &arguments)
            .await
        {
            Ok(result) => Ok(CallToolResult::structured(result)),
            Err(error) => Err(McpError::internal_error(
                "call http error",
                Some(Value::String(error.to_string())),
            )),
        }
    }

    pub async fn execute_tool_call_from_id(
        &self,
        endpoint_id: Uuid,
        tool_name: &str,
        arguments: &Value,
    ) -> anyhow::Result<Value> {
        match self.get_endpoint(endpoint_id).await {
            Ok(endpoint) => {
                self.execute_tool_call(&endpoint, tool_name, arguments)
                    .await
            }
            Err(error) => Err(Error::from(error).context("Failed to execute tool call")),
        }
    }

    pub async fn get_endpoint(&self, endpoint_id: Uuid) -> anyhow::Result<Endpoint> {
        let endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE id = ?"
        )
            .bind(endpoint_id.to_string())
            .fetch_one(DB_POOL.get().expect("DB_POOL not initialized"))
            .await?;

        Ok(endpoint)
    }

    pub async fn execute_tool_call(
        &self,
        endpoint: &Endpoint,
        tool_name: &str,
        arguments: &Value,
    ) -> anyhow::Result<Value> {
        tracing::info!(
            "Executing tool call: {} for endpoint: {}",
            tool_name,
            endpoint.name
        );
        tracing::debug!("Arguments: {}", arguments);

        // Parse swagger content to get API specifications
        let swagger_spec: crate::models::SwaggerSpec =
            serde_json::from_str(&endpoint.swagger_content)?;

        // Parse tool name to extract method, path and operation info
        let (method, path, operation) = parse_tool_name(&swagger_spec, tool_name)?;

        // Build the base URL from swagger spec
        let base_url = build_base_url(&swagger_spec)?;

        // Build the full URL with path parameters
        let full_url = build_url(&base_url, &path, arguments)?;

        // Extract query parameters, headers, and body from arguments based on Swagger spec
        let (query_params, headers, body) = extract_request_parts(arguments, &operation)?;

        tracing::info!("Making HTTP request to: {}", full_url);
        tracing::debug!(
            "Method: {}, Query params: {:?}, Headers: {:?}, Body: {:?}",
            method,
            query_params,
            headers,
            body
        );

        // Make the HTTP request
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&full_url),
            "POST" => self.http_client.post(&full_url),
            "PUT" => self.http_client.put(&full_url),
            "DELETE" => self.http_client.delete(&full_url),
            "PATCH" => self.http_client.patch(&full_url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        // Add query parameters
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add body for POST/PUT/PATCH requests
        if let Some(body_data) = body {
            tracing::debug!(
                "Request body: {}",
                serde_json::to_string_pretty(&body_data)?
            );
            request = request.json(&body_data);
        }

        // Execute the request
        let response = request.send().await?;
        let status = response.status();
        let response_text = response.text().await?;

        tracing::info!("Received response with status: {}", status);
        tracing::debug!("Response body: {}", response_text);

        // Update metrics
        let pool = DB_POOL.get().expect("DB_POOL not initialized");
        update_metrics(pool, endpoint.id, status.is_success()).await?;

        // Format response
        let response_value = match serde_json::from_str::<Value>(&response_text) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!("Failed to parse response as JSON: {}", e);
                Value::String(response_text.clone())
            }
        };

        let result = json!({
            "status": status.as_u16(),
            "success": status.is_success(),
            "response": response_value
        });

        tracing::info!(
            "Tool call result: {}",
            serde_json::to_string_pretty(&result)?
        );
        Ok(result)
    }
}

impl ServerHandler for Adapter {
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "str:////Users/to/some/path/" => {
                let cwd = "/Users/to/some/path/";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(cwd, uri)],
                })
            }
            "memo://insights" => {
                let memo = "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...";
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(memo, uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": uri
                })),
            )),
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        self.inner_call_tool(request, context)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        tracing::info!("context: {:?}", context);
        self.inner_list_tools(context)
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            // todo: 替换成对应endpoint的描述
            instructions: Some("This server provides swagger http tools.".to_string()),
        }
    }
}
