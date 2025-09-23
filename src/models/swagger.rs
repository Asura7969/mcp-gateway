use crate::models::Endpoint;
use rmcp::model::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwaggerSpec {
    pub openapi: String,
    pub info: Info,
    pub servers: Option<Vec<Server>>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<RequestBody>,
    pub responses: Option<HashMap<String, Response>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub required: Option<bool>,
    pub description: Option<String>,
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwaggerToMcpRequest {
    pub swagger_content: String,
    pub endpoint_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwaggerToMcpResponse {
    pub endpoint_id: uuid::Uuid,
    pub mcp_config: crate::models::McpConfig,
    pub tools: Vec<McpTool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(rename = "outputSchema")]
    pub output_schema: Option<serde_json::Value>,
}

impl From<&McpTool> for Tool {
    fn from(mcp_tool: &McpTool) -> Self {
        let out = match mcp_tool.output_schema {
            Some(ref o) => Some(Arc::new(o.as_object().unwrap().clone())),
            _ => None,
        };
        Tool {
            name: std::borrow::Cow::Owned(format!("{}", mcp_tool.name)),
            description: Some(std::borrow::Cow::Owned(format!("{}", mcp_tool.description))),
            input_schema: Arc::new(mcp_tool.input_schema.as_object().unwrap().clone()),
            output_schema: out,
            annotations: None,
        }
    }
}
