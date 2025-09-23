use crate::models::{DbPool, McpTool, SwaggerSpec};
use anyhow::anyhow;
use serde_json::Value;
use uuid::Uuid;

pub fn generate_mcp_tools(spec: &SwaggerSpec) -> anyhow::Result<Vec<McpTool>> {
    let mut tools = Vec::new();

    for (path, path_item) in &spec.paths {
        // Generate tools for each HTTP method
        if let Some(operation) = &path_item.get {
            tools.push(create_mcp_tool("GET", path, operation, spec)?);
        }
        if let Some(operation) = &path_item.post {
            tools.push(create_mcp_tool("POST", path, operation, spec)?);
        }
        if let Some(operation) = &path_item.put {
            tools.push(create_mcp_tool("PUT", path, operation, spec)?);
        }
        if let Some(operation) = &path_item.delete {
            tools.push(create_mcp_tool("DELETE", path, operation, spec)?);
        }
        if let Some(operation) = &path_item.patch {
            tools.push(create_mcp_tool("PATCH", path, operation, spec)?);
        }
    }

    Ok(tools)
}

pub fn create_mcp_tool(
    method: &str,
    path: &str,
    operation: &crate::models::Operation,
    spec: &SwaggerSpec, // Add spec parameter
) -> anyhow::Result<McpTool> {
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

    let title = operation
        .summary
        .clone()
        .unwrap_or_else(|| format!("{} {}", method, path));

    let description = operation
        .description
        .clone()
        .or_else(|| operation.summary.clone())
        .unwrap_or_else(|| format!("{} API for {}", method, path));

    // Build input schema
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    // Add path parameters
    if let Some(parameters) = &operation.parameters {
        for param in parameters {
            if param.location == "path" {
                properties.insert(
                    param.name.clone(),
                    serde_json::json!({
                        "type": "string",
                        "description": param.description.clone().unwrap_or_default()
                    }),
                );
                if param.required.unwrap_or(false) {
                    required.push(param.name.clone());
                }
            }
            if param.location == "query" {
                let param_type = param
                    .schema
                    .as_ref()
                    .and_then(|s| s.schema_type.clone())
                    .unwrap_or_else(|| "string".to_string());

                properties.insert(
                    param.name.clone(),
                    serde_json::json!({
                        "type": param_type,
                        "description": param.description.clone().unwrap_or_default()
                    }),
                );
                if param.required.unwrap_or(false) {
                    required.push(param.name.clone());
                }
            }
        }
    }

    // Add request body if present
    if let Some(request_body) = &operation.request_body {
        if let Some(content) = request_body.content.get("application/json") {
            if let Some(schema) = &content.schema {
                // Instead of wrapping in "body", directly expand the schema properties
                let body_schema = schema_to_json_schema(schema, spec)?;
                if let Some(body_properties) =
                    body_schema.get("properties").and_then(|p| p.as_object())
                {
                    // Insert all properties from the body schema directly
                    for (key, value) in body_properties {
                        properties.insert(key.clone(), value.clone());
                    }

                    // Handle required fields from the body schema
                    if let Some(body_required) =
                        body_schema.get("required").and_then(|r| r.as_array())
                    {
                        for req_field in body_required {
                            if let Some(req_str) = req_field.as_str() {
                                required.push(req_str.to_string());
                            }
                        }
                    }
                } else {
                    // For simple types or schemas without properties, insert directly without "body" wrapper
                    // Add a property with a descriptive name based on the schema type
                    let property_name = if let Some(schema_type) = &schema.schema_type {
                        match schema_type.as_str() {
                            "string" => "input".to_string(),
                            "number" => "value".to_string(),
                            "integer" => "value".to_string(),
                            "boolean" => "flag".to_string(),
                            "array" => "items".to_string(),
                            _ => "data".to_string(),
                        }
                    } else {
                        "data".to_string()
                    };

                    properties.insert(property_name.clone(), body_schema);
                    if request_body.required.unwrap_or(false) {
                        required.push(property_name);
                    }
                }
            }
        }
    }

    // Create input schema - use default empty object if no properties
    let input_schema = if properties.is_empty() {
        serde_json::json!({
            "type": "object",
            "title": "EmptyObject",
            "description": ""
        })
    } else {
        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    };

    // Build output schema from responses
    let output_schema = if let Some(responses) = &operation.responses {
        // Look for 200 response first, then any 2xx response
        let response_schema = if let Some(ok_response) = responses.get("200") {
            extract_response_schema(ok_response, spec)
        } else {
            // Find first 2xx response
            responses
                .iter()
                .find(|(code, _)| code.starts_with("2"))
                .and_then(|(_, response)| extract_response_schema(response, spec))
        };

        response_schema
    } else {
        None
    };

    Ok(McpTool {
        name: tool_name,
        title,
        description,
        input_schema,
        output_schema,
    })
}

pub fn schema_to_json_schema(
    schema: &crate::models::Schema,
    spec: &SwaggerSpec,
) -> anyhow::Result<Value> {
    // Handle $ref references
    if let Some(reference) = &schema.reference {
        // 解析引用，例如 "#/components/schemas/BotAgentDto"
        if reference.starts_with("#/components/schemas/") {
            let schema_name = &reference["#/components/schemas/".len()..];
            // 从组件中查找引用的模式
            if let Some(components) = &spec.components {
                if let Some(schemas) = &components.schemas {
                    if let Some(referenced_schema) = schemas.get(schema_name) {
                        // 递归解析引用的模式
                        return schema_to_json_schema(referenced_schema, spec);
                    }
                }
            }
        }
        // 如果无法解析引用，返回包含引用信息的对象
        return Ok(serde_json::json!({
            "$ref": reference
        }));
    }

    let mut json_schema = serde_json::Map::new();

    if let Some(schema_type) = &schema.schema_type {
        json_schema.insert("type".to_string(), Value::String(schema_type.clone()));
    }

    if let Some(format) = &schema.format {
        json_schema.insert("format".to_string(), Value::String(format.clone()));
    }

    if let Some(description) = &schema.description {
        json_schema.insert(
            "description".to_string(),
            Value::String(description.clone()),
        );
    }

    if let Some(properties) = &schema.properties {
        let mut props = serde_json::Map::new();
        for (key, prop_schema) in properties {
            props.insert(key.clone(), schema_to_json_schema(prop_schema, spec)?);
        }
        json_schema.insert("properties".to_string(), Value::Object(props));
    }

    if let Some(items) = &schema.items {
        json_schema.insert("items".to_string(), schema_to_json_schema(items, spec)?);
    }

    if let Some(required) = &schema.required {
        json_schema.insert(
            "required".to_string(),
            Value::Array(required.iter().map(|r| Value::String(r.clone())).collect()),
        );
    }

    Ok(Value::Object(json_schema))
}

pub async fn update_metrics(pool: &DbPool, endpoint_id: Uuid, success: bool) -> anyhow::Result<()> {
    let error_increment = if success { 0 } else { 1 };
    sqlx::query(
        "UPDATE endpoint_metrics SET
             request_count = request_count + 1,
             response_count = response_count + 1,
             error_count = error_count + ?
             WHERE endpoint_id = ?",
    )
    .bind(error_increment)
    .bind(endpoint_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub fn extract_request_parts(
    arguments: &Value,
    operation: &crate::models::Operation,
) -> anyhow::Result<(Vec<(String, String)>, Vec<(String, String)>, Option<Value>)> {
    let mut query_params = Vec::new();
    let mut headers = Vec::new();
    let mut body = None;

    // 根据Swagger规范中的参数定义来组织参数
    if let Some(parameters) = &operation.parameters {
        for param in parameters {
            let param_name = &param.name;

            // 从arguments中查找对应的参数值
            if let Some(param_value) = arguments.get(param_name) {
                match param.location.as_str() {
                    "query" => {
                        if let Some(value_str) = param_value.as_str() {
                            query_params.push((param_name.clone(), value_str.to_string()));
                        } else if let Some(value_num) = param_value.as_number() {
                            query_params.push((param_name.clone(), value_num.to_string()));
                        } else if let Some(value_bool) = param_value.as_bool() {
                            query_params.push((param_name.clone(), value_bool.to_string()));
                        }
                    }
                    "header" => {
                        if let Some(value_str) = param_value.as_str() {
                            headers.push((param_name.clone(), value_str.to_string()));
                        }
                    }
                    "path" => {
                        // 路径参数在URL构建时处理，这里不需要添加到请求参数中
                    }
                    _ => {
                        // 对于其他位置的参数，暂时忽略
                    }
                }
            }
        }
    }

    // 对于POST/PUT/PATCH请求，处理请求体
    if let Some(request_body) = &operation.request_body {
        // 检查arguments中是否有body字段
        if let Some(body_value) = arguments.get("body") {
            body = Some(body_value.clone());
        } else {
            // 根据requestBody的schema定义来确定请求体内容
            if let Some(content) = request_body.content.values().next() {
                if let Some(schema) = &content.schema {
                    if let Some(properties) = &schema.properties {
                        // 创建请求体对象，只包含schema中定义的属性
                        let mut body_obj = serde_json::Map::new();

                        for (prop_name, _) in properties {
                            // 从arguments中查找对应的属性值
                            if let Some(prop_value) = arguments.get(prop_name) {
                                body_obj.insert(prop_name.clone(), prop_value.clone());
                            }
                        }

                        // 检查是否有必需的字段未提供
                        if let Some(required_fields) = &schema.required {
                            for required_field in required_fields {
                                if !body_obj.contains_key(required_field) {
                                    // 检查arguments中是否有这个必需字段
                                    if let Some(required_value) = arguments.get(required_field) {
                                        body_obj
                                            .insert(required_field.clone(), required_value.clone());
                                    }
                                }
                            }
                        }

                        if !body_obj.is_empty() {
                            body = Some(Value::Object(body_obj));
                        }
                    }
                }
            }

            // 如果还是没有构建出请求体，尝试使用不在参数定义中的字段
            if body.is_none() {
                let mut body_obj = serde_json::Map::new();
                if let Some(args_obj) = arguments.as_object() {
                    for (key, value) in args_obj {
                        // 检查这个参数是否已经在path/query/header中处理过
                        let mut already_processed = false;

                        if let Some(parameters) = &operation.parameters {
                            for param in parameters {
                                if &param.name == key {
                                    already_processed = true;
                                    break;
                                }
                            }
                        }

                        // 也要排除特殊字段
                        if key == "body" {
                            already_processed = true;
                        }

                        if !already_processed {
                            // 确保key是字符串类型
                            body_obj.insert(key.clone(), value.clone());
                        }
                    }
                }

                if !body_obj.is_empty() {
                    body = Some(Value::Object(body_obj));
                }
            }
        }
    } else {
        // 对于GET/DELETE等没有请求体的方法，确保body为None
        body = None;
    }

    // Add default content-type for JSON if we have a body
    if body.is_some() {
        headers.push(("Content-Type".to_string(), "application/json".to_string()));
    }

    Ok((query_params, headers, body))
}

pub fn build_url(base_url: &str, path: &str, arguments: &Value) -> anyhow::Result<String> {
    let mut url_path = path.to_string();

    // Replace path parameters from the arguments object directly
    // Path parameters are those that are part of the path template like /users/{id}
    if let Some(args_obj) = arguments.as_object() {
        // Find placeholders in the path like {id}
        let placeholders: Vec<_> = url_path
            .match_indices('{')
            .filter_map(|(start, _)| {
                if let Some(end) = url_path[start..].find('}') {
                    Some((start, start + end + 1))
                } else {
                    None
                }
            })
            .collect();

        // Replace each placeholder with the corresponding argument value
        for (start, end) in placeholders.iter().rev() {
            let placeholder = &url_path[*start..*end]; // e.g., "{id}"
            let param_name = &placeholder[1..placeholder.len() - 1]; // e.g., "id"

            if let Some(param_value) = args_obj.get(param_name) {
                if let Some(value_str) = param_value.as_str() {
                    url_path.replace_range(*start..*end, value_str);
                } else if let Some(value_num) = param_value.as_number() {
                    url_path.replace_range(*start..*end, &value_num.to_string());
                } else if let Some(value_bool) = param_value.as_bool() {
                    url_path.replace_range(*start..*end, &value_bool.to_string());
                }
            }
        }
    }

    Ok(format!("{}{}", base_url.trim_end_matches('/'), url_path))
}

pub fn build_base_url(swagger_spec: &crate::models::SwaggerSpec) -> anyhow::Result<String> {
    // Build base URL from swagger spec
    // For OpenAPI 3.x, use servers array
    if let Some(servers) = &swagger_spec.servers {
        if let Some(server) = servers.get(0) {
            return Ok(server.url.clone());
        }
    }

    // Fallback to localhost
    Ok("http://localhost:8080".to_string())
}

pub fn parse_tool_name<'a>(
    swagger_spec: &'a SwaggerSpec,
    tool_name: &str,
) -> anyhow::Result<(String, String, &'a crate::models::Operation)> {
    // Find the operation that matches this tool name
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
                let expected_tool_name = operation.operation_id.clone().unwrap_or_else(|| {
                    format!(
                        "{}_{}_api",
                        method.to_lowercase(),
                        path.replace('/', "_")
                            .replace('{', "")
                            .replace('}', "")
                            .trim_start_matches('_')
                    )
                });

                if expected_tool_name == tool_name {
                    return Ok((method.to_string(), path.clone(), operation));
                }
            }
        }
    }

    Err(anyhow!("Tool not found: {}", tool_name))
}

pub fn extract_response_schema(
    response: &crate::models::Response,
    spec: &SwaggerSpec,
) -> Option<serde_json::Value> {
    if let Some(content) = &response.content {
        if let Some(media_type) = content.get("application/json") {
            if let Some(schema) = &media_type.schema {
                match schema_to_json_schema(schema, spec) {
                    Ok(json_schema) => return Some(json_schema),
                    Err(_) => return None,
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SwaggerSpec;

    #[test]
    fn test_generate_mcp_tools_with_body_unwrapping() -> anyhow::Result<()> {
        let spec: SwaggerSpec = serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test": {
      "post": {
        "summary": "Test endpoint with body",
        "operationId": "testBody",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "required": ["name", "email"],
                "properties": {
                  "name": {
                    "type": "string",
                    "description": "User name"
                  },
                  "email": {
                    "type": "string",
                    "description": "User email"
                  },
                  "age": {
                    "type": "integer",
                    "description": "User age"
                  }
                }
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"###,
        )?;

        let tools = generate_mcp_tools(&spec)?;
        assert_eq!(tools.len(), 1);

        let tool = &tools[0];
        assert_eq!(tool.name, "testBody");

        // Print the entire input schema for debugging
        println!("Input schema: {:#}", tool.input_schema);

        // Check that input schema is properly structured
        let input_schema = &tool.input_schema;
        assert_eq!(input_schema["type"], "object");

        // Check that properties exist and are properly expanded (not wrapped in "body")
        let properties = input_schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("email"));
        assert!(properties.contains_key("age"));

        // Check property details (without description as it's not available in Schema struct)
        println!("Name property: {:#}", properties["name"]);
        assert_eq!(properties["name"]["type"], "string");
        assert_eq!(properties["email"]["type"], "string");
        assert_eq!(properties["age"]["type"], "integer");

        // Check required fields
        let required = input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::Value::String("name".to_string())));
        assert!(required.contains(&serde_json::Value::String("email".to_string())));

        Ok(())
    }

    #[test]
    fn test_generate_mcp_tools_with_simple_body() -> anyhow::Result<()> {
        let spec: SwaggerSpec = serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test": {
      "post": {
        "summary": "Test endpoint with simple body",
        "operationId": "testSimpleBody",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "string",
                "description": "Simple string body"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"###,
        )?;

        let tools = generate_mcp_tools(&spec)?;
        assert_eq!(tools.len(), 1);

        let tool = &tools[0];
        assert_eq!(tool.name, "testSimpleBody");

        // Check that input schema is properly structured
        let input_schema = &tool.input_schema;
        assert_eq!(input_schema["type"], "object");

        // For simple types, it should use a descriptive name instead of "body"
        let properties = input_schema["properties"].as_object().unwrap();
        assert!(properties.contains_key("input"));
        assert_eq!(properties["input"]["type"], "string");

        Ok(())
    }

    #[test]
    fn test_generate_mcp_tools_with_various_simple_types() -> anyhow::Result<()> {
        // Test with number type
        let spec_number: SwaggerSpec = serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test-number": {
      "post": {
        "summary": "Test endpoint with number body",
        "operationId": "testNumberBody",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "number"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"###,
        )?;

        let tools_number = generate_mcp_tools(&spec_number)?;
        let tool_number = &tools_number[0];
        let properties_number = tool_number.input_schema["properties"].as_object().unwrap();
        assert!(properties_number.contains_key("value"));
        assert_eq!(properties_number["value"]["type"], "number");

        // Test with boolean type
        let spec_boolean: SwaggerSpec = serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test-boolean": {
      "post": {
        "summary": "Test endpoint with boolean body",
        "operationId": "testBooleanBody",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "boolean"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"###,
        )?;

        let tools_boolean = generate_mcp_tools(&spec_boolean)?;
        let tool_boolean = &tools_boolean[0];
        let properties_boolean = tool_boolean.input_schema["properties"].as_object().unwrap();
        assert!(properties_boolean.contains_key("flag"));
        assert_eq!(properties_boolean["flag"]["type"], "boolean");

        // Test with array type
        let spec_array: SwaggerSpec = serde_json::from_str(
            r###"{
  "openapi": "3.1.0",
  "info": {
    "title": "Test API",
    "version": "1.0.0"
  },
  "paths": {
    "/test-array": {
      "post": {
        "summary": "Test endpoint with array body",
        "operationId": "testArrayBody",
        "requestBody": {
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "type": "array",
                "items": {
                  "type": "string"
                }
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Success"
          }
        }
      }
    }
  }
}"###,
        )?;

        let tools_array = generate_mcp_tools(&spec_array)?;
        let tool_array = &tools_array[0];
        let properties_array = tool_array.input_schema["properties"].as_object().unwrap();
        assert!(properties_array.contains_key("items"));
        assert_eq!(properties_array["items"]["type"], "array");

        Ok(())
    }
}
