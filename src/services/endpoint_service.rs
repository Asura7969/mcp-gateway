use crate::models::{
    CreateEndpointRequest, DbPool, Endpoint, EndpointDetailResponse, EndpointMetrics,
    EndpointResponse, EndpointStatus, McpConfig, UpdateEndpointRequest,
};
use crate::services::EndpointEvent;
use crate::utils::{generate_api_details, get_china_time};
use anyhow::Result;
use serde_json::Value;
use sqlx::Row;
use std::convert::TryInto;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EndpointService {
    pool: DbPool,
    event_sender: mpsc::Sender<EndpointEvent>,
}

impl EndpointService {
    pub fn new(pool: DbPool, event_sender: mpsc::Sender<EndpointEvent>) -> Self {
        Self { pool, event_sender }
    }

    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn create_endpoint(
        &self,
        request: CreateEndpointRequest,
    ) -> Result<EndpointResponse> {
        // First, check if an endpoint with the same name already exists
        let existing_endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE name = ?"
        )
            .bind(&request.name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(endpoint) = existing_endpoint {
            // If endpoint with same name exists, merge the data instead of creating new one
            tracing::info!(
                "Endpoint with name '{}' already exists, merging data",
                request.name
            );

            // Parse the existing and new swagger content
            let existing_swagger: Value = serde_json::from_str(&endpoint.swagger_content)?;
            let new_swagger: Value = serde_json::from_str(&request.swagger_content)?;

            // Merge the swagger specifications
            let merged_swagger = self.merge_swagger_specs(existing_swagger, new_swagger)?;

            // Update the existing endpoint with merged data
            let now = get_china_time();
            sqlx::query(
                "UPDATE endpoints SET description = COALESCE(?, description), swagger_content = ?, updated_at = ? WHERE id = ?"
            )
                .bind(&request.description)
                .bind(serde_json::to_string(&merged_swagger)?)
                .bind(now)
                .bind(endpoint.id.to_string())
                .execute(&self.pool)
                .await?;

            // Update API paths table with new paths
            self.update_api_paths_table(endpoint.id, &merged_swagger)
                .await?;

            let updated_endpoint = self.get_endpoint_by_id(endpoint.id).await?;
            self.event_sender
                .send(EndpointEvent::UPDATE(endpoint.name))
                .await?;
            Ok(updated_endpoint.into())
        } else {
            // Create new endpoint
            let id = Uuid::new_v4();
            let now = get_china_time();

            let _endpoint_result = sqlx::query(
                r#"
                INSERT INTO endpoints (id, name, description, swagger_content, status, created_at, updated_at, connection_count)
                VALUES (?, ?, ?, ?, 'stopped', ?, ?, 0)
                "#,
            )
                .bind(id.to_string())
                .bind(&request.name)
                .bind(&request.description)
                .bind(&request.swagger_content)
                .bind(now)
                .bind(now)
                .execute(&self.pool)
                .await?;

            // Parse swagger content and populate API paths table
            let swagger_spec: Value = serde_json::from_str(&request.swagger_content)?;
            self.update_api_paths_table(id, &swagger_spec).await?;

            let endpoint = self.get_endpoint_by_id(id).await?;

            self.event_sender
                .send(EndpointEvent::Created(endpoint.name.clone()))
                .await?;

            Ok(endpoint.into())
        }
    }

    /// Merge two swagger specifications, avoiding duplicate paths and methods
    fn merge_swagger_specs(&self, existing: Value, new: Value) -> Result<Value> {
        let mut merged = existing.clone();

        // Get paths from both specs
        if let (Some(existing_paths), Some(new_paths)) = (
            merged.get_mut("paths").and_then(|v| v.as_object_mut()),
            new.get("paths").and_then(|v| v.as_object()),
        ) {
            // Merge paths, avoiding duplicates
            for (path, new_path_item) in new_paths {
                if !existing_paths.contains_key(path) {
                    // New path, add it
                    existing_paths.insert(path.clone(), new_path_item.clone());
                } else {
                    // Existing path, merge methods
                    if let (Some(existing_path_item), Some(new_path_item)) = (
                        existing_paths.get_mut(path).and_then(|v| v.as_object_mut()),
                        new_path_item.as_object(),
                    ) {
                        // Merge HTTP methods
                        for (method, new_operation) in new_path_item {
                            if method.to_uppercase() != *method {
                                // Skip non-HTTP methods
                                continue;
                            }

                            if !existing_path_item.contains_key(method) {
                                // New method for this path, add it
                                existing_path_item.insert(method.clone(), new_operation.clone());
                            } else {
                                // Method already exists, we'll keep the existing one
                                tracing::info!(
                                    "Skipping duplicate method {} for path {}",
                                    method,
                                    path
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(merged)
    }

    /// Update the api_paths table with paths and methods from swagger spec
    async fn update_api_paths_table(&self, endpoint_id: Uuid, swagger_spec: &Value) -> Result<()> {
        // Clear existing entries for this endpoint
        sqlx::query("DELETE FROM api_paths WHERE endpoint_id = ?")
            .bind(endpoint_id.to_string())
            .execute(&self.pool)
            .await?;

        // Extract paths and methods from swagger spec
        if let Some(paths) = swagger_spec.get("paths").and_then(|v| v.as_object()) {
            for (path, path_item) in paths {
                if let Some(path_item_obj) = path_item.as_object() {
                    // Process each HTTP method for this path
                    for (method, operation) in path_item_obj {
                        if method.to_uppercase() != *method {
                            // Skip non-HTTP methods
                            continue;
                        }

                        let operation_id = operation
                            .get("operationId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let summary = operation
                            .get("summary")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let description = operation
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        // Insert the API path entry
                        let api_path_id = Uuid::new_v4();
                        sqlx::query(
                            "INSERT INTO api_paths (id, endpoint_id, path, method, operation_id, summary, description) VALUES (?, ?, ?, ?, ?, ?, ?)"
                        )
                            .bind(api_path_id.to_string())
                            .bind(endpoint_id.to_string())
                            .bind(path)
                            .bind(method.to_uppercase())
                            .bind(operation_id)
                            .bind(summary)
                            .bind(description)
                            .execute(&self.pool)
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn get_endpoints(&self) -> Result<Vec<EndpointResponse>> {
        let endpoints = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(endpoints.into_iter().map(|e| e.into()).collect())
    }

    /// Get all endpoints with full data (including swagger_content)
    pub async fn get_all_endpoints(&self) -> Result<Vec<Endpoint>> {
        let endpoints = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(endpoints)
    }

    /// Get endpoints with pagination, search and filter support
    pub async fn get_endpoints_paginated(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
        search: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<EndpointResponse>, u64)> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let offset = (page - 1) * page_size;

        // Build the base query
        let mut where_conditions: Vec<String> = vec![];
        let mut params: Vec<String> = vec![];

        // Add search condition
        if let Some(search_term) = search {
            if !search_term.trim().is_empty() {
                where_conditions.push("(name LIKE ? OR description LIKE ?)".to_string());
                let search_pattern = format!("%{}%", search_term);
                params.push(search_pattern.clone());
                params.push(search_pattern);
            }
        }

        // Add status filter
        if let Some(status) = status_filter {
            if !status.trim().is_empty() && status.to_lowercase() != "all" {
                where_conditions.push("status = ?".to_string());
                params.push(status.to_lowercase());
            }
        }

        // Build WHERE clause
        let (_where_clause, count_query, query) = if where_conditions.is_empty() {
            (
                String::new(),
                "SELECT COUNT(*) as total FROM endpoints".to_string(),
                "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints ORDER BY created_at DESC LIMIT ? OFFSET ?".to_string(),
            )
        } else {
            let where_clause = where_conditions.join(" AND ");
            (
                where_clause.clone(),
                format!("SELECT COUNT(*) as total FROM endpoints WHERE {}", where_clause),
                format!("SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE {} ORDER BY created_at DESC LIMIT ? OFFSET ?", where_clause),
            )
        };

        // Count total records
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let count_result = count_query_builder.fetch_one(&self.pool).await?;
        let total: i64 = count_result.get("total");

        // Fetch paginated results

        let mut query_builder = sqlx::query_as::<_, Endpoint>(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(page_size).bind(offset);

        let endpoints = query_builder.fetch_all(&self.pool).await?;

        Ok((
            endpoints.into_iter().map(|e| e.into()).collect(),
            total as u64,
        ))
    }

    pub async fn get_endpoint_by_id(&self, id: Uuid) -> Result<Endpoint> {
        let endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE id = ?"
        )
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Endpoint not found"))?;

        Ok(endpoint)
    }

    pub async fn get_endpoint_by_name(&self, name: String) -> Result<Endpoint> {
        let endpoint = sqlx::query_as::<_, Endpoint>(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE name = ?"
        )
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        Ok(endpoint)
    }

    pub async fn get_endpoint_by_names(&self, names: Vec<String>) -> Result<Vec<Endpoint>> {
        if names.is_empty() {
            return Ok(vec![]);
        }

        // 构建IN子句的占位符
        let placeholders: Vec<String> = (1..=names.len()).map(|i| format!("${}", i)).collect();
        let in_clause = placeholders.join(", ");

        let query = format!(
            "SELECT id, name, description, swagger_content, status, created_at, updated_at, connection_count FROM endpoints WHERE name IN ({})",
            in_clause
        );

        let mut query_builder = sqlx::query_as::<_, Endpoint>(&query);

        // 绑定每个名称参数
        for name in names {
            query_builder = query_builder.bind(name);
        }

        let endpoints = query_builder.fetch_all(&self.pool).await?;
        Ok(endpoints)
    }

    pub async fn get_endpoint_detail(&self, id: Uuid) -> Result<EndpointDetailResponse> {
        let endpoint = self.get_endpoint_by_id(id).await?;

        // Parse swagger content
        tracing::debug!("Parsing swagger content for endpoint: {}", endpoint.name);
        tracing::debug!("Swagger content length: {}", endpoint.swagger_content.len());

        let swagger_spec: crate::models::SwaggerSpec =
            match serde_json::from_str(&endpoint.swagger_content) {
                Ok(spec) => {
                    tracing::debug!("Successfully parsed swagger spec");
                    spec
                }
                Err(e) => {
                    tracing::error!("Failed to parse swagger content: {}", e);
                    tracing::error!("Swagger content: {}", &endpoint.swagger_content);
                    return Err(e.into());
                }
            };

        // Generate API details
        let api_details = generate_api_details(&swagger_spec)?;

        // Get base URL
        let base_url = swagger_spec
            .servers
            .as_ref()
            .and_then(|servers| servers.first())
            .map(|server| server.url.clone());

        // Generate MCP config
        let mcp_config = McpConfig {
            server_name: format!("mcp-{}", endpoint.name),
            command: vec!["mcp-gateway".to_string()],
            args: vec!["--endpoint-id".to_string(), id.to_string()],
        };

        // 尝试序列化swagger_spec，添加错误处理
        let swagger_spec_value = match serde_json::to_value(&swagger_spec) {
            Ok(value) => {
                tracing::debug!("Successfully serialized swagger spec to JSON value");
                value
            }
            Err(e) => {
                tracing::error!("Failed to serialize swagger spec to JSON value: {}", e);
                // 记录swagger_spec的详细信息以帮助调试
                tracing::error!("Swagger spec debug: {:#?}", swagger_spec);
                return Err(e.into());
            }
        };

        Ok(EndpointDetailResponse {
            id: endpoint.id,
            name: endpoint.name,
            description: endpoint.description,
            status: endpoint.status,
            created_at: endpoint.created_at,
            updated_at: endpoint.updated_at,
            connection_count: endpoint.connection_count,
            swagger_spec: swagger_spec_value,
            mcp_config,
            api_details,
            base_url,
        })
    }

    pub async fn update_endpoint(
        &self,
        id: Uuid,
        request: UpdateEndpointRequest,
    ) -> Result<EndpointResponse> {
        let mut query = "UPDATE endpoints SET updated_at = ?".to_string();
        let mut params: Vec<String> = vec![get_china_time().to_rfc3339()];

        if let Some(name) = &request.name {
            query.push_str(", name = ?");
            params.push(name.clone());
        }

        if let Some(description) = &request.description {
            query.push_str(", description = ?");
            params.push(description.clone());
        }

        if let Some(swagger_content) = &request.swagger_content {
            query.push_str(", swagger_content = ?");
            params.push(swagger_content.clone());
        }

        if let Some(status) = &request.status {
            query.push_str(", status = ?");
            params.push(match status {
                EndpointStatus::Running => "running".to_string(),
                EndpointStatus::Stopped => "stopped".to_string(),
                EndpointStatus::Deleted => "deleted".to_string(),
            });
        }

        query.push_str(" WHERE id = ?");
        params.push(id.to_string());

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        query_builder.execute(&self.pool).await?;

        let endpoint = self.get_endpoint_by_id(id).await?;
        self.event_sender
            .send(EndpointEvent::UPDATE(endpoint.name.clone()))
            .await?;
        Ok(endpoint.into())
    }

    pub async fn delete_endpoint(&self, id: Uuid) -> Result<()> {
        match self.get_endpoint_by_id(id).await {
            Ok(endpoint) => {
                // 物理删除端点记录
                sqlx::query("DELETE FROM endpoints WHERE id = ?")
                    .bind(id.to_string())
                    .execute(&self.pool)
                    .await?;
                self.event_sender
                    .send(EndpointEvent::DELETE(endpoint.name))
                    .await?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    pub async fn get_endpoint_metrics(&self, id: Uuid) -> Result<EndpointMetrics> {
        let metrics = sqlx::query(
            "SELECT endpoint_id, request_count, response_count, error_count, avg_response_time, current_connections, total_connection_time FROM endpoint_metrics WHERE endpoint_id = ?"
        )
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = metrics {
            // Handle DECIMAL to f64 conversion
            let avg_response_time: rust_decimal::Decimal = row.get("avg_response_time");
            let avg_response_time_f64: f64 = avg_response_time.try_into().unwrap_or(0.0);

            Ok(EndpointMetrics {
                endpoint_id: id,
                request_count: row.get::<u64, _>("request_count"),
                response_count: row.get::<u64, _>("response_count"),
                error_count: row.get::<u64, _>("error_count"),
                avg_response_time: avg_response_time_f64,
                current_connections: row.get::<i32, _>("current_connections"),
                total_connection_time: row.get::<u64, _>("total_connection_time"),
            })
        } else {
            // Create default metrics if not exists
            let metrics_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO endpoint_metrics (id, endpoint_id, request_count, response_count, error_count, avg_response_time, current_connections, total_connection_time) VALUES (?, ?, 0, 0, 0, 0.0, 0, 0)"
            )
                .bind(metrics_id.to_string())
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;

            Ok(EndpointMetrics {
                endpoint_id: id,
                request_count: 0,
                response_count: 0,
                error_count: 0,
                avg_response_time: 0.0,
                current_connections: 0,
                total_connection_time: 0,
            })
        }
    }

    /// Get metrics for all endpoints
    pub async fn get_all_endpoint_metrics(&self) -> Result<Vec<EndpointMetrics>> {
        // First get all active endpoint IDs
        let endpoint_ids = sqlx::query("SELECT id FROM endpoints")
            .fetch_all(&self.pool)
            .await?;

        let mut all_metrics = Vec::new();

        for row in endpoint_ids {
            let endpoint_id_str: String = row.get("id");
            let endpoint_id = Uuid::parse_str(&endpoint_id_str)?;

            match self.get_endpoint_metrics(endpoint_id).await {
                Ok(metrics) => all_metrics.push(metrics),
                Err(e) => {
                    tracing::warn!("Failed to get metrics for endpoint {}: {}", endpoint_id, e);
                    // Continue with other endpoints even if one fails
                }
            }
        }

        Ok(all_metrics)
    }

    /// Start an endpoint (set status to running)
    pub async fn start_endpoint(&self, id: Uuid) -> Result<()> {
        // Verify endpoint exists and is not deleted
        let endpoint = self.get_endpoint_by_id(id).await?;

        if endpoint.status == EndpointStatus::Deleted {
            return Err(anyhow::anyhow!("Cannot start deleted endpoint"));
        }

        if endpoint.status == EndpointStatus::Running {
            return Err(anyhow::anyhow!("Endpoint is already running"));
        }

        // Validate swagger content before starting
        let _: serde_json::Value = serde_json::from_str(&endpoint.swagger_content)
            .map_err(|e| anyhow::anyhow!("Invalid swagger content: {}", e))?;

        sqlx::query("UPDATE endpoints SET status = 'running', updated_at = ? WHERE id = ?")
            .bind(get_china_time())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        tracing::info!("Started endpoint: {} ({})", endpoint.name, id);
        Ok(())
    }

    /// Stop an endpoint (set status to stopped)
    pub async fn stop_endpoint(&self, id: Uuid) -> Result<()> {
        // Verify endpoint exists and is not deleted
        let endpoint = self.get_endpoint_by_id(id).await?;

        if endpoint.status == EndpointStatus::Deleted {
            return Err(anyhow::anyhow!("Cannot stop deleted endpoint"));
        }

        if endpoint.status == EndpointStatus::Stopped {
            return Err(anyhow::anyhow!("Endpoint is already stopped"));
        }

        sqlx::query("UPDATE endpoints SET status = 'stopped', updated_at = ? WHERE id = ?")
            .bind(get_china_time())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        tracing::info!("Stopped endpoint: {} ({})", endpoint.name, id);
        Ok(())
    }

    pub async fn sync_endpoint_vector(&self, name: String) -> Result<()> {
        let r = self.event_sender.send(EndpointEvent::UPDATE(name)).await?;
        Ok(r)
    }

    pub async fn update_connection_count(&self, id: Uuid, delta: i32) -> Result<()> {
        sqlx::query("UPDATE endpoints SET connection_count = connection_count + ? WHERE id = ?")
            .bind(delta)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateEndpointRequest, EndpointStatus};

    async fn create_test_pool() -> DbPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "mysql://mcpuser:mcppassword@localhost:3306/mcp_gateway_test".to_string()
        });

        sqlx::MySqlPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    #[ignore] // 需要测试数据库
    async fn test_create_endpoint() {
        let (tx, _rx) = mpsc::channel(100);
        let pool = create_test_pool().await;
        let service = EndpointService::new(pool, tx);

        let request = CreateEndpointRequest {
            name: "Test Endpoint".to_string(),
            description: Some("A test endpoint".to_string()),
            swagger_content: r#"{"openapi":"3.0.0"}"#.to_string(),
        };

        let result = service.create_endpoint(request).await;
        assert!(result.is_ok());

        let endpoint = result.unwrap();
        assert_eq!(endpoint.name, "Test Endpoint");
        assert_eq!(endpoint.status, EndpointStatus::Stopped);
    }

    #[tokio::test]
    #[ignore] // 需要测试数据库
    async fn test_create_endpoint_with_same_name_merges_data() {
        let (tx, _rx) = mpsc::channel(100);
        let pool = create_test_pool().await;
        let service = EndpointService::new(pool, tx);

        // 创建第一个端点
        let request1 = CreateEndpointRequest {
            name: "Merge Test Endpoint".to_string(),
            description: Some("First endpoint".to_string()),
            swagger_content:
                r#"{"openapi":"3.0.0", "paths": {"/test1": {"get": {"summary": "Test 1"}}}}"#
                    .to_string(),
        };

        let result1 = service.create_endpoint(request1).await;
        assert!(result1.is_ok());
        let endpoint1 = result1.unwrap();

        // 尝试创建同名端点，应该合并数据
        let request2 = CreateEndpointRequest {
            name: "Merge Test Endpoint".to_string(),
            description: Some("Second endpoint".to_string()),
            swagger_content:
                r#"{"openapi":"3.0.0", "paths": {"/test2": {"post": {"summary": "Test 2"}}}}"#
                    .to_string(),
        };

        let result2 = service.create_endpoint(request2).await;
        assert!(result2.is_ok());
        let endpoint2 = result2.unwrap();

        // 验证是同一个端点（ID相同）
        assert_eq!(endpoint1.id, endpoint2.id);

        // 验证描述被更新（使用新值）
        assert_eq!(endpoint2.description, Some("Second endpoint".to_string()));

        // 获取详细信息以检查Swagger内容
        let detail = service.get_endpoint_detail(endpoint2.id).await.unwrap();

        // 验证Swagger内容被合并
        let paths = detail
            .swagger_spec
            .get("paths")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(paths.contains_key("/test1"));
        assert!(paths.contains_key("/test2"));
    }

    #[tokio::test]
    #[ignore] // 需要测试数据库
    async fn test_merge_swagger_specs_no_duplicates() {
        let (tx, _rx) = mpsc::channel(100);
        let pool = create_test_pool().await;
        let service = EndpointService::new(pool, tx);

        let existing =
            serde_json::from_str(r#"{"paths": {"/test": {"get": {"summary": "Existing"}}}}"#)
                .unwrap();
        let new =
            serde_json::from_str(r#"{"paths": {"/test2": {"post": {"summary": "New"}}}}"#).unwrap();

        let merged = service.merge_swagger_specs(existing, new).unwrap();
        let paths = merged.get("paths").unwrap().as_object().unwrap();

        // 应该包含两个路径
        assert_eq!(paths.len(), 2);
        assert!(paths.contains_key("/test"));
        assert!(paths.contains_key("/test2"));
    }

    #[tokio::test]
    #[ignore] // 需要测试数据库
    async fn test_merge_swagger_specs_with_duplicates() {
        let (tx, _rx) = mpsc::channel(100);
        let pool = create_test_pool().await;
        let service = EndpointService::new(pool, tx);

        let existing =
            serde_json::from_str(r#"{"paths": {"/test": {"get": {"summary": "Existing"}}}}"#)
                .unwrap();
        let new =
            serde_json::from_str(r#"{"paths": {"/test": {"post": {"summary": "New"}}}}"#).unwrap();

        let merged = service.merge_swagger_specs(existing, new).unwrap();
        let paths = merged.get("paths").unwrap().as_object().unwrap();
        let test_path = paths.get("/test").unwrap().as_object().unwrap();

        // 应该包含两个方法：get（来自existing）和post（来自new）
        assert_eq!(test_path.len(), 2);
        assert!(test_path.contains_key("get"));
        assert!(test_path.contains_key("post"));
    }
}
