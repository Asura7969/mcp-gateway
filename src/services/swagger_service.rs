use crate::models::{
    ApiDetail, ApiParameter, CreateEndpointRequest, McpConfig, SwaggerSpec,
    SwaggerToMcpRequest, SwaggerToMcpResponse,
};
use crate::services::EndpointService;
use crate::utils::{generate_mcp_tools, schema_to_json_schema};
use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

pub struct SwaggerService {
    endpoint_service: EndpointService,
}

impl SwaggerService {
    pub fn new(endpoint_service: EndpointService) -> Self {
        Self { endpoint_service }
    }

    pub async fn convert_swagger_to_mcp(
        &self,
        request: SwaggerToMcpRequest,
    ) -> Result<SwaggerToMcpResponse> {
        // Parse swagger content
        let swagger_spec: SwaggerSpec = if request.swagger_content.trim().starts_with('{') {
            serde_json::from_str(&request.swagger_content)?
        } else {
            serde_yaml::from_str(&request.swagger_content)?
        };

        // Validate swagger spec
        self.validate_swagger_spec(&swagger_spec)?;

        // Check if any paths and methods already exist for this endpoint name
        let existing_endpoint = sqlx::query(
            "SELECT id, name, swagger_content FROM endpoints WHERE name = ? AND status != 'deleted'"
        )
        .bind(&request.endpoint_name)
        .fetch_optional(self.endpoint_service.get_pool())
        .await?;

        let endpoint_response = if let Some(row) = existing_endpoint {
            // Endpoint exists, check for duplicate paths and methods
            let endpoint_id_str: String = row.get("id");
            let _endpoint_id = Uuid::parse_str(&endpoint_id_str)?;
            let existing_swagger_content: String = row.get("swagger_content");

            let existing_swagger: Value = serde_json::from_str(&existing_swagger_content)?;
            let new_swagger: Value = serde_json::to_value(&swagger_spec)?;

            // Check for duplicate paths and methods
            self.check_for_duplicate_paths(&existing_swagger, &new_swagger)?;

            // Since no duplicates were found, we can proceed with creating the endpoint
            // The endpoint service will handle merging the data
            let create_request = CreateEndpointRequest {
                name: request.endpoint_name.clone(),
                description: request.description.clone(),
                swagger_content: request.swagger_content,
            };

            self.endpoint_service
                .create_endpoint(create_request)
                .await?
        } else {
            // Create new endpoint
            let create_request = CreateEndpointRequest {
                name: request.endpoint_name.clone(),
                description: request.description.clone(),
                swagger_content: request.swagger_content,
            };

            self.endpoint_service
                .create_endpoint(create_request)
                .await?
        };

        // Generate MCP tools from swagger paths
        let tools = generate_mcp_tools(&swagger_spec)?;

        // Generate MCP config
        let mcp_config = McpConfig {
            server_name: format!("mcp-{}", request.endpoint_name),
            command: vec!["mcp-gateway".to_string()],
            args: vec![
                "--endpoint-id".to_string(),
                endpoint_response.id.to_string(),
            ],
        };

        Ok(SwaggerToMcpResponse {
            endpoint_id: endpoint_response.id,
            mcp_config,
            tools,
        })
    }

    /// Check for duplicate paths and methods between two swagger specs
    fn check_for_duplicate_paths(&self, existing: &Value, new: &Value) -> Result<()> {
        if let (Some(existing_paths), Some(new_paths)) = (
            existing.get("paths").and_then(|v| v.as_object()),
            new.get("paths").and_then(|v| v.as_object()),
        ) {
            for (path, new_path_item) in new_paths {
                if let Some(existing_path_item) = existing_paths.get(path) {
                    // Path exists in both specs, check methods
                    if let (Some(existing_methods), Some(new_methods)) =
                        (existing_path_item.as_object(), new_path_item.as_object())
                    {
                        for (method, _) in new_methods {
                            // Convert method to uppercase for comparison
                            let upper_method = method.to_uppercase();

                            // Only check HTTP methods
                            if [
                                "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "TRACE",
                            ]
                            .contains(&upper_method.as_str())
                            {
                                if existing_methods.contains_key(&upper_method)
                                    || existing_methods.contains_key(method)
                                {
                                    // Duplicate path and method found
                                    return Err(anyhow!(
                                        "API path '{}' with method '{}' already exists",
                                        path,
                                        upper_method
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_swagger_spec(&self, spec: &SwaggerSpec) -> Result<()> {
        if spec.openapi.is_empty() {
            return Err(anyhow!("OpenAPI version is required"));
        }

        if !spec.openapi.starts_with("3.") {
            return Err(anyhow!("Only OpenAPI 3.x is supported"));
        }

        if spec.paths.is_empty() {
            return Err(anyhow!("At least one path is required"));
        }

        Ok(())
    }

    /// Generate API details from swagger spec
    pub fn generate_api_details(&self, spec: &SwaggerSpec) -> Result<Vec<ApiDetail>> {
        let mut api_details = Vec::new();
        let base_url = spec
            .servers
            .as_ref()
            .and_then(|servers| servers.first())
            .map(|server| server.url.clone());

        for (path, path_item) in &spec.paths {
            // Generate details for each HTTP method
            if let Some(operation) = &path_item.get {
                api_details.push(self.create_api_detail("GET", path, operation, spec, &base_url)?);
            }
            if let Some(operation) = &path_item.post {
                api_details.push(self.create_api_detail("POST", path, operation, spec, &base_url)?);
            }
            if let Some(operation) = &path_item.put {
                api_details.push(self.create_api_detail("PUT", path, operation, spec, &base_url)?);
            }
            if let Some(operation) = &path_item.delete {
                api_details
                    .push(self.create_api_detail("DELETE", path, operation, spec, &base_url)?);
            }
            if let Some(operation) = &path_item.patch {
                api_details
                    .push(self.create_api_detail("PATCH", path, operation, spec, &base_url)?);
            }
        }

        Ok(api_details)
    }

    fn create_api_detail(
        &self,
        method: &str,
        path: &str,
        operation: &crate::models::Operation,
        spec: &SwaggerSpec,
        _base_url: &Option<String>,
    ) -> Result<ApiDetail> {
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        let mut request_body_schema = None;
        let mut response_schema = None;

        // Process parameters
        if let Some(parameters) = &operation.parameters {
            for param in parameters {
                let api_param = ApiParameter {
                    name: param.name.clone(),
                    required: param.required.unwrap_or(false),
                    description: param.description.clone(),
                    param_type: param
                        .schema
                        .as_ref()
                        .and_then(|s| s.schema_type.clone())
                        .unwrap_or_else(|| "string".to_string()),
                    schema: param
                        .schema
                        .as_ref()
                        .map(|s| schema_to_json_schema(s, spec))
                        .transpose()?,
                };

                match param.location.as_str() {
                    "path" => path_params.push(api_param),
                    "query" => query_params.push(api_param),
                    "header" => header_params.push(api_param),
                    _ => {} // Ignore other parameter types for now
                }
            }
        }

        // Process request body
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = request_body.content.get("application/json") {
                if let Some(schema) = &content.schema {
                    request_body_schema = Some(schema_to_json_schema(schema, spec)?);
                }
            }
        }

        // Process responses
        let responses = serde_json::to_value(&operation.responses)?;

        // Process response schema (use first 2xx response)
        if let Some(responses_map) = &operation.responses {
            for (status_code, response) in responses_map {
                if status_code.starts_with("2") {
                    if let Some(content) = &response.content {
                        if let Some(media_type) = content.get("application/json") {
                            if let Some(schema) = &media_type.schema {
                                response_schema = Some(schema_to_json_schema(schema, spec)?);
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(ApiDetail {
            path: path.to_string(),
            method: method.to_string(),
            summary: operation.summary.clone(),
            description: operation.description.clone(),
            operation_id: operation.operation_id.clone(),
            path_params,
            query_params,
            header_params,
            request_body_schema,
            response_schema,
            responses,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SwaggerToMcpRequest;

    fn create_test_swagger_spec() -> SwaggerSpec {
        serde_json::from_str(
            r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/test": {
                    "get": {
                        "summary": "Test endpoint",
                        "operationId": "getTest",
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            }
        }"#,
        )
        .unwrap()
    }

    fn create_optimized_swagger_spec() -> SwaggerSpec {
        // 从文件中读取优化后的JSON内容
        serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "agent-bot",
    "description": "机器人接口",
    "version": "1.0.0"
  },
  "paths": {
    "/bot-agent/findByAgentId": {
      "get": {
        "summary": "机器人查询接口",
        "description": "根据AgentId查询机器人信息",
        "operationId": "findByAgentId",
        "parameters": [
          {
            "name": "agentId",
            "in": "query",
            "description": "agentId",
            "required": true,
            "schema": {
              "type": "string"
            }
          }
        ],
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ResultBoolean"
                }
              }
            }
          }
        }
      }
    },
    "/bot-agent/save": {
      "post": {
        "summary": "保存机器人-agent关系",
        "description": "保存机器人与agent的关系",
        "operationId": "saveBotAgent",
        "requestBody": {
          "description": "机器人Agent信息",
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/BotAgentDto"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/schemas/ResultBoolean"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "BotAgentDto": {
        "type": "object",
        "required": [
          "appId",
          "appSecret",
          "agentId",
          "agentApiKey"
        ],
        "properties": {
          "agentApiKey": {
            "type": "string",
            "description": "API密钥"
          },
          "agentId": {
            "type": "string",
            "description": "Agent ID"
          },
          "appEncryptKey": {
            "type": "string",
            "description": "应用加密密钥"
          },
          "appId": {
            "type": "string",
            "description": "应用ID"
          },
          "appSecret": {
            "type": "string",
            "description": "应用密钥"
          },
          "appVerificationToken": {
            "type": "string",
            "description": "应用验证令牌"
          },
          "createTime": {
            "type": "string",
            "description": "创建时间"
          },
          "updateTime": {
            "type": "string",
            "description": "更新时间"
          }
        }
      },
      "ResultBoolean": {
        "type": "object",
        "properties": {
          "code": {
            "type": "integer",
            "description": "状态码"
          },
          "data": {
            "type": "boolean",
            "description": "数据"
          },
          "msg": {
            "type": "string",
            "description": "消息"
          },
          "success": {
            "type": "boolean",
            "description": "是否成功"
          },
          "timestamp": {
            "type": "integer",
            "format": "int64",
            "description": "时间戳"
          }
        }
      }
    }
  }
}"###,
        )
        .unwrap()
    }

    fn create_no_params_swagger_spec() -> SwaggerSpec {
        serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "description": "测试接口",
    "version": "1.0.0"
  },
  "servers": [
    {
      "url": "http://test-service.dev.starcharge.cloud"
    }
  ],
  "paths": {
    "/test/ping": {
      "get": {
        "summary": "Ping接口",
        "description": "测试连通性",
        "operationId": "ping",
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "message": {
                      "type": "string",
                      "description": "响应消息"
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"###,
        )
        .unwrap()
    }

    fn create_array_type_swagger_spec() -> SwaggerSpec {
        serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Array Type Test API",
    "description": "测试数组类型字段接口",
    "version": "1.0.0"
  },
  "servers": [
    {
      "url": "http://array-test-service.dev.starcharge.cloud"
    }
  ],
  "paths": {
    "/users": {
      "get": {
        "summary": "获取用户列表",
        "description": "返回用户列表信息，包含复杂对象数组",
        "operationId": "getUsers",
        "responses": {
          "200": {
            "description": "成功响应",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "users": {
                      "type": "array",
                      "description": "用户列表",
                      "items": {
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "integer",
                            "description": "用户ID"
                          },
                          "name": {
                            "type": "string",
                            "description": "用户姓名"
                          },
                          "email": {
                            "type": "string",
                            "description": "用户邮箱"
                          },
                          "tags": {
                            "type": "array",
                            "description": "用户标签",
                            "items": {
                              "type": "string",
                              "description": "标签名称"
                            }
                          },
                          "profile": {
                            "type": "object",
                            "description": "用户资料",
                            "properties": {
                              "age": {
                                "type": "integer",
                                "description": "年龄"
                              },
                              "address": {
                                "type": "string",
                                "description": "地址"
                              },
                              "skills": {
                                "type": "array",
                                "description": "技能列表",
                                "items": {
                                  "type": "object",
                                  "properties": {
                                    "name": {
                                      "type": "string",
                                      "description": "技能名称"
                                    },
                                    "level": {
                                      "type": "string",
                                      "description": "技能等级",
                                      "enum": ["beginner", "intermediate", "advanced"]
                                    }
                                  },
                                  "required": ["name", "level"]
                                }
                              }
                            },
                            "required": ["age"]
                          }
                        },
                        "required": ["id", "name"]
                      }
                    },
                    "total": {
                      "type": "integer",
                      "description": "总用户数"
                    }
                  },
                  "required": ["users", "total"]
                }
              }
            }
          }
        }
      },
      "post": {
        "summary": "创建用户",
        "description": "创建新用户，请求体包含数组类型字段",
        "operationId": "createUser",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "name": {
                    "type": "string",
                    "description": "用户姓名"
                  },
                  "emails": {
                    "type": "array",
                    "description": "用户邮箱列表",
                    "items": {
                      "type": "string",
                      "description": "邮箱地址"
                    }
                  },
                  "preferences": {
                    "type": "array",
                    "description": "用户偏好设置",
                    "items": {
                      "type": "object",
                      "properties": {
                        "category": {
                          "type": "string",
                          "description": "偏好类别"
                        },
                        "enabled": {
                          "type": "boolean",
                          "description": "是否启用"
                        }
                      },
                      "required": ["category", "enabled"]
                    }
                  }
                },
                "required": ["name", "emails"]
              }
            }
          }
        },
        "responses": {
          "201": {
            "description": "用户创建成功",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "integer",
                      "description": "用户ID"
                    },
                    "message": {
                      "type": "string",
                      "description": "成功消息"
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"###,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_validate_swagger_spec() {
        let pool = sqlx::MySqlPool::connect_lazy("mysql://test").unwrap();
        let endpoint_service = EndpointService::new(pool);
        let service = SwaggerService::new(endpoint_service);

        let spec = create_test_swagger_spec();
        assert!(service.validate_swagger_spec(&spec).is_ok());

        // Test invalid spec
        let mut invalid_spec = spec.clone();
        invalid_spec.openapi = "2.0".to_string();
        assert!(service.validate_swagger_spec(&invalid_spec).is_err());
    }

    #[tokio::test]
    async fn test_generate_mcp_tools() {
        let spec = create_test_swagger_spec();
        let tools = generate_mcp_tools(&spec).unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "getTest");
        assert_eq!(tools[0].description, "Test endpoint");
    }

    #[tokio::test]
    async fn test_generate_mcp_tools_with_optimized_schema() {
        let spec = create_optimized_swagger_spec();
        let tools = generate_mcp_tools(&spec).unwrap();

        // 验证生成的工具数量
        assert_eq!(tools.len(), 2);

        // 验证 findByAgentId GET 工具
        let find_tool = tools.iter().find(|t| t.name == "findByAgentId").unwrap();
        assert_eq!(find_tool.title, "机器人查询接口");
        assert_eq!(find_tool.description, "根据AgentId查询机器人信息");

        // 验证 inputSchema 结构
        let input_schema = &find_tool.input_schema;
        assert_eq!(input_schema["type"], "object");
        assert!(input_schema["properties"].as_object().is_some());
        assert!(input_schema["required"].as_array().is_some());

        // 验证查询参数
        let properties = input_schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("agentId"));
        assert_eq!(properties["agentId"]["type"], "string");
        assert_eq!(properties["agentId"]["description"], "agentId");

        // 验证 required 字段
        let required = input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("agentId".to_string())));

        // 验证 outputSchema 是否存在
        assert!(find_tool.output_schema.is_some());
        let output_schema = find_tool.output_schema.as_ref().unwrap();
        assert_eq!(output_schema["type"], "object");
        assert!(output_schema["properties"].as_object().is_some());

        // 验证 saveBotAgent POST 工具
        let save_tool = tools.iter().find(|t| t.name == "saveBotAgent").unwrap();
        assert_eq!(save_tool.title, "保存机器人-agent关系");
        assert_eq!(save_tool.description, "保存机器人与agent的关系");

        // 验证 inputSchema 结构
        let input_schema = &save_tool.input_schema;
        assert_eq!(input_schema["type"], "object");
        assert!(input_schema["properties"].as_object().is_some());

        // 验证 body 参数被展开而不是包装在"body"中
        let properties = input_schema["properties"].as_object().unwrap();
        // 检查BotAgentDto的属性是否被正确展开
        assert!(properties.contains_key("agentId"));
        assert!(properties.contains_key("appId"));
        assert!(properties.contains_key("appSecret"));
        assert!(properties.contains_key("agentApiKey"));

        // 验证属性类型和描述
        assert_eq!(properties["agentId"]["type"], "string");
        assert_eq!(properties["agentId"]["description"], "Agent ID");
        assert_eq!(properties["appId"]["type"], "string");
        assert_eq!(properties["appId"]["description"], "应用ID");
        assert_eq!(properties["appSecret"]["type"], "string");
        assert_eq!(properties["appSecret"]["description"], "应用密钥");
        assert_eq!(properties["agentApiKey"]["type"], "string");
        assert_eq!(properties["agentApiKey"]["description"], "API密钥");

        // 验证其他属性的描述
        assert_eq!(properties["appEncryptKey"]["description"], "应用加密密钥");
        assert_eq!(
            properties["appVerificationToken"]["description"],
            "应用验证令牌"
        );
        assert_eq!(properties["createTime"]["description"], "创建时间");
        assert_eq!(properties["updateTime"]["description"], "更新时间");

        // 验证 required 字段
        let required = input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("agentId".to_string())));
        assert!(required.contains(&serde_json::Value::String("appId".to_string())));
        assert!(required.contains(&serde_json::Value::String("appSecret".to_string())));
        assert!(required.contains(&serde_json::Value::String("agentApiKey".to_string())));

        // 验证 outputSchema 是否存在
        assert!(save_tool.output_schema.is_some());
        let output_schema = save_tool.output_schema.as_ref().unwrap();
        assert_eq!(output_schema["type"], "object");
        assert!(output_schema["properties"].as_object().is_some());
    }

    #[tokio::test]
    async fn test_generate_mcp_tools_with_no_params() {
        let spec = create_no_params_swagger_spec();
        let tools = generate_mcp_tools(&spec).unwrap();

        // 验证生成的工具数量
        assert_eq!(tools.len(), 1);

        // 验证 ping GET 工具
        let ping_tool = tools.iter().find(|t| t.name == "ping").unwrap();
        assert_eq!(ping_tool.title, "Ping接口");
        assert_eq!(ping_tool.description, "测试连通性");

        // 验证 inputSchema 使用默认值
        let input_schema = &ping_tool.input_schema;
        assert_eq!(input_schema["type"], "object");
        assert_eq!(input_schema["title"], "EmptyObject");
        assert_eq!(input_schema["description"], "");

        // 验证没有 properties 和 required 字段
        assert!(input_schema.get("properties").is_none());
        assert!(input_schema.get("required").is_none());

        // 验证 outputSchema 是否存在
        assert!(ping_tool.output_schema.is_some());
        let output_schema = ping_tool.output_schema.as_ref().unwrap();
        assert_eq!(output_schema["type"], "object");
        assert!(output_schema["properties"].as_object().is_some());
    }

    #[tokio::test]
    async fn test_check_for_duplicate_paths_no_duplicates() {
        let pool = sqlx::MySqlPool::connect_lazy("mysql://test").unwrap();
        let endpoint_service = EndpointService::new(pool);
        let service = SwaggerService::new(endpoint_service);

        let existing =
            serde_json::from_str(r#"{"paths": {"/test1": {"get": {"summary": "Test 1"}}}}"#)
                .unwrap();
        let new = serde_json::from_str(r#"{"paths": {"/test2": {"post": {"summary": "Test 2"}}}}"#)
            .unwrap();

        // 应该没有重复路径
        assert!(service.check_for_duplicate_paths(&existing, &new).is_ok());
    }

    #[tokio::test]
    async fn test_check_for_duplicate_paths_with_duplicates() {
        let pool = sqlx::MySqlPool::connect_lazy("mysql://test").unwrap();
        let endpoint_service = EndpointService::new(pool);
        let service = SwaggerService::new(endpoint_service);

        let existing =
            serde_json::from_str(r#"{"paths": {"/test": {"get": {"summary": "Existing"}}}}"#)
                .unwrap();
        let new =
            serde_json::from_str(r#"{"paths": {"/test": {"get": {"summary": "New"}}}}"#).unwrap();

        // 应该检测到重复路径
        let result = service.check_for_duplicate_paths(&existing, &new);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_generate_api_details_with_array_types() {
        let pool = sqlx::MySqlPool::connect_lazy("mysql://test").unwrap();
        let endpoint_service = EndpointService::new(pool);
        let service = SwaggerService::new(endpoint_service);

        let spec = create_array_type_swagger_spec();
        let api_details = service.generate_api_details(&spec).unwrap();

        // 验证生成的API详情数量
        assert_eq!(api_details.len(), 2); // GET和POST两个方法

        // 验证GET方法详情
        let get_users = api_details.iter().find(|d| d.method == "GET").unwrap();
        assert_eq!(get_users.path, "/users");
        assert_eq!(get_users.summary, Some("获取用户列表".to_string()));
        
        // 验证响应体包含数组类型字段
        assert!(get_users.response_schema.is_some());
        let response_schema = get_users.response_schema.as_ref().unwrap();
        let properties = response_schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("users"));
        
        let users_field = &properties["users"];
        assert_eq!(users_field["type"], "array");
        assert!(users_field["items"].as_object().is_some());
        
        // 验证数组项的对象属性
        let item_properties = users_field["items"]["properties"].as_object().unwrap();
        assert!(item_properties.contains_key("id"));
        assert!(item_properties.contains_key("name"));
        assert!(item_properties.contains_key("profile"));
        
        // 验证嵌套的对象字段
        let profile_field = &item_properties["profile"];
        assert_eq!(profile_field["type"], "object");
        let profile_properties = profile_field["properties"].as_object().unwrap();
        assert!(profile_properties.contains_key("skills"));
        
        // 验证嵌套的数组字段
        let skills_field = &profile_properties["skills"];
        assert_eq!(skills_field["type"], "array");
        assert!(skills_field["items"].as_object().is_some());

        // 验证POST方法详情
        let create_user = api_details.iter().find(|d| d.method == "POST").unwrap();
        assert_eq!(create_user.path, "/users");
        assert_eq!(create_user.summary, Some("创建用户".to_string()));
        
        // 验证请求体包含数组类型字段
        assert!(create_user.request_body_schema.is_some());
        let request_schema = create_user.request_body_schema.as_ref().unwrap();
        let req_properties = request_schema["properties"].as_object().unwrap();
        assert!(req_properties.contains_key("emails"));
        assert!(req_properties.contains_key("preferences"));
        
        let emails_field = &req_properties["emails"];
        assert_eq!(emails_field["type"], "array");
        assert_eq!(emails_field["items"]["type"], "string");
        
        let preferences_field = &req_properties["preferences"];
        assert_eq!(preferences_field["type"], "array");
        assert!(preferences_field["items"].as_object().is_some());
    }

    #[tokio::test]
    async fn test_property_descriptions_in_schema() {
        let spec = create_optimized_swagger_spec();
        let tools = generate_mcp_tools(&spec).unwrap();

        // 验证 saveBotAgent 工具
        let save_tool = tools.iter().find(|t| t.name == "saveBotAgent").unwrap();
        let properties = save_tool.input_schema["properties"].as_object().unwrap();

        // 验证所有属性都有正确的描述
        assert_eq!(properties["agentId"]["description"], "Agent ID");
        assert_eq!(properties["appId"]["description"], "应用ID");
        assert_eq!(properties["appSecret"]["description"], "应用密钥");
        assert_eq!(properties["agentApiKey"]["description"], "API密钥");
        assert_eq!(properties["appEncryptKey"]["description"], "应用加密密钥");
        assert_eq!(
            properties["appVerificationToken"]["description"],
            "应用验证令牌"
        );
        assert_eq!(properties["createTime"]["description"], "创建时间");
        assert_eq!(properties["updateTime"]["description"], "更新时间");
    }
}
