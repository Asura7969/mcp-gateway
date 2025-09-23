use crate::models::SwaggerSpec;
use crate::utils::generate_mcp_tools;
use chrono::{DateTime, Utc};
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    #[serde(with = "uuid_as_string")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub swagger_content: String,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connection_count: i32,
}

impl From<&Endpoint> for Vec<Tool> {
    fn from(endpoint: &Endpoint) -> Vec<Tool> {
        let spec: SwaggerSpec = serde_json::from_str(endpoint.swagger_content.as_str()).unwrap();
        let tools = generate_mcp_tools(&spec).unwrap();
        tools.iter().map(Tool::from).collect::<Vec<_>>()
    }
}

// Custom UUID serialization for database compatibility
mod uuid_as_string {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

// Custom FromRow implementation for database compatibility
impl FromRow<'_, sqlx::mysql::MySqlRow> for Endpoint {
    fn from_row(row: &sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID format: {}", e).into()))?;

        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "running" => EndpointStatus::Running,
            "stopped" => EndpointStatus::Stopped,
            "deleted" => EndpointStatus::Deleted,
            _ => {
                return Err(sqlx::Error::Decode(
                    format!("Invalid status: {}", status_str).into(),
                ))
            }
        };

        Ok(Self {
            id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            swagger_content: row.try_get("swagger_content")?,
            status,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            connection_count: row.try_get("connection_count")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "endpoint_status", rename_all = "lowercase")]
pub enum EndpointStatus {
    Running,
    Stopped,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEndpointRequest {
    pub name: String,
    pub description: Option<String>,
    pub swagger_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEndpointRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub swagger_content: Option<String>,
    pub status: Option<EndpointStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connection_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connection_count: i32,
    pub swagger_spec: serde_json::Value,
    pub mcp_config: McpConfig,
    pub api_details: Vec<ApiDetail>,
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfig {
    pub server_name: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDetail {
    pub path: String,
    pub method: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub path_params: Vec<ApiParameter>,
    pub query_params: Vec<ApiParameter>,
    pub header_params: Vec<ApiParameter>,
    pub request_body_schema: Option<serde_json::Value>,
    pub response_schema: Option<serde_json::Value>,
    pub responses: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiParameter {
    pub name: String,
    pub required: bool,
    pub description: Option<String>,
    pub param_type: String,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub endpoint_id: Uuid,
    pub request_count: u64,
    pub response_count: u64,
    pub error_count: u64,
    pub avg_response_time: f64,
    pub current_connections: i32,
    pub total_connection_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedEndpointsResponse {
    pub endpoints: Vec<EndpointResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct EndpointQueryParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub search: Option<String>,
    pub status: Option<String>,
}

impl From<Endpoint> for EndpointResponse {
    fn from(endpoint: Endpoint) -> Self {
        Self {
            id: endpoint.id,
            name: endpoint.name,
            description: endpoint.description,
            status: endpoint.status,
            created_at: endpoint.created_at,
            updated_at: endpoint.updated_at,
            connection_count: endpoint.connection_count,
        }
    }
}
