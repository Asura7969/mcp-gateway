use crate::models::{DbPool, Endpoint};
use crate::utils::{
    build_base_url, build_url, extract_request_parts, parse_tool_name, update_metrics,
};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct McpService {
    pool: DbPool,
    http_client: Client,
}

impl McpService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            http_client: Client::new(),
        }
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

                    // Build input schema (simplified)
                    let input_schema = serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
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

    pub async fn execute_tool_call(
        &self,
        endpoint: &Endpoint,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<String> {
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
        update_metrics(&self.pool, endpoint.id, status.is_success()).await?;

        // Format response
        let response_value = match serde_json::from_str::<Value>(&response_text) {
            Ok(parsed) => parsed,
            Err(e) => {
                tracing::warn!("Failed to parse response as JSON: {}", e);
                Value::String(response_text.clone())
            }
        };

        let result = serde_json::json!({
            "status": status.as_u16(),
            "success": status.is_success(),
            "response": response_value
        });

        tracing::info!(
            "Tool call result: {}",
            serde_json::to_string_pretty(&result)?
        );

        // 添加额外的调试信息来检查result的结构
        if let Some(obj) = result.as_object() {
            for (key, value) in obj {
                tracing::debug!(
                    "Result key: '{}' (type: {:?}), value: {:?}",
                    key,
                    key,
                    value
                );
            }
        }

        // 在序列化之前添加调试信息，检查result结构
        match serde_json::to_string_pretty(&result) {
            Ok(json_string) => Ok(json_string),
            Err(e) => {
                tracing::error!("Failed to serialize result to JSON: {}", e);
                tracing::error!("Result structure: {:?}", result);
                // 返回一个简化版本的响应
                let simplified_result = serde_json::json!({
                    "status": status.as_u16(),
                    "success": status.is_success(),
                    "response": "Response serialization error occurred"
                });
                Ok(serde_json::to_string(&simplified_result)?)
            }
        }
    }

    pub async fn get_endpoint(&self, endpoint_id: Uuid) -> Result<Endpoint> {
        let endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE id = ?"
        )
        .bind(endpoint_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(endpoint)
    }

    pub async fn get_endpoints(&self) -> Result<Vec<Endpoint>> {
        let endpoints = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE status != 'deleted' ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(endpoints)
    }
}
