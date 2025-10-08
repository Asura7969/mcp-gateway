#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use utoipa::ToSchema;

    /// 接口节点 - 表示一个API接口
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct ApiInterface {
        /// 接口路径，如 /api/users/{id}
        pub path: String,
        /// HTTP方法，如 GET, POST, PUT, DELETE
        pub method: String,
        /// 接口名称/标题
        pub name: String,
        /// 接口描述
        pub description: Option<String>,
        /// 请求参数定义
        pub parameters: Vec<Parameter>,
        /// 响应字段定义
        pub responses: Vec<ResponseField>,
        /// 接口标签/分类
        pub tags: Vec<String>,
        /// 业务领域，如 user, order, product
        pub domain: Option<String>,
        /// 是否已弃用
        pub deprecated: bool,
    }

    /// 参数定义
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct Parameter {
        /// 参数名称
        pub name: String,
        /// 参数类型：path, query, body, header
        pub param_type: String,
        /// 数据类型：string, integer, boolean, object, array
        pub data_type: String,
        /// 是否必需
        pub required: bool,
        /// 参数描述
        pub description: Option<String>,
        /// 示例值
        pub example: Option<String>,
        /// 如果是对象或数组，包含嵌套字段
        pub nested_fields: Option<Vec<NestedField>>,
    }

    /// 响应字段定义
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct ResponseField {
        /// 字段名称
        pub name: String,
        /// 数据类型
        pub data_type: String,
        /// 字段描述
        pub description: Option<String>,
        /// 示例值
        pub example: Option<String>,
        /// 如果是对象或数组，包含嵌套字段
        pub nested_fields: Option<Vec<NestedField>>,
    }

    /// 嵌套字段定义（用于复杂对象）
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct NestedField {
        /// 字段名称
        pub name: String,
        /// 数据类型
        pub data_type: String,
        /// 字段描述
        pub description: Option<String>,
        /// 示例值
        pub example: Option<String>,
    }

    /// 接口依赖关系 - 表示接口之间的关系
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct InterfaceDependency {
        /// 源接口ID
        pub from_interface: String,
        /// 目标接口ID
        pub to_interface: String,
        /// 依赖类型
        pub dependency_type: DependencyType,
        /// 字段映射关系
        pub field_mappings: Vec<FieldMapping>,
        /// 依赖描述
        pub description: Option<String>,
        /// 依赖强度：1-10，数字越大表示依赖越强
        pub strength: u8,
    }

    /// 依赖类型枚举
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub enum DependencyType {
        /// 数据依赖：需要先调用A接口获取数据，再调用B接口
        DataDependency,
        /// 业务流程依赖：按业务逻辑顺序调用
        BusinessFlow,
        /// 参数传递：A接口的输出作为B接口的输入
        ParameterPassing,
        /// 认证依赖：需要先通过认证接口
        AuthDependency,
        /// 验证依赖：需要先验证某些条件
        ValidationDependency,
        /// 相关接口：功能相关但不强制依赖
        Related,
    }

    /// 字段映射关系
    #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
    pub struct FieldMapping {
        /// 源字段路径，如 response.data.user_id
        pub source_field: String,
        /// 目标字段路径，如 parameters.id
        pub target_field: String,
        /// 映射类型：direct, transform, conditional
        pub mapping_type: String,
        /// 转换规则（可选）
        pub transform_rule: Option<String>,
    }

    /// 存储接口关系请求
    #[derive(Debug, Serialize, Deserialize, ToSchema)]
    pub struct StoreInterfaceRelationRequest {
        /// 接口列表
        pub interfaces: Vec<ApiInterface>,
        /// 依赖关系列表
        pub dependencies: Vec<InterfaceDependency>,
        /// 项目ID
        pub project_id: String,
        /// 版本号（可选）
        pub version: Option<String>,
    }

    /// 查询接口请求
    #[derive(Debug, Serialize, Deserialize, ToSchema)]
    pub struct QueryInterfaceRequest {
        /// 查询关键词
        pub query: String,
        /// 项目ID（可选，用于过滤）
        pub project_id: Option<String>,
        /// 最大返回结果数（可选，默认10）
        pub max_results: Option<u32>,
        /// 是否包含依赖关系（可选，默认true）
        pub include_dependencies: Option<bool>,
    }

    #[test]
    fn test_interface_model_creation() {
        let interface = ApiInterface {
            path: "/api/users".to_string(),
            method: "GET".to_string(),
            name: "Get Users".to_string(),
            description: Some("Get all users".to_string()),
            parameters: vec![],
            responses: vec![],
            tags: vec!["user".to_string()],
            domain: Some("user".to_string()),
            deprecated: false,
        };
        
        assert_eq!(interface.path, "/api/users");
        assert_eq!(interface.method, "GET");
        assert_eq!(interface.name, "Get Users");
        assert!(!interface.deprecated);
    }

    #[test]
    fn test_dependency_model_creation() {
        let dependency = InterfaceDependency {
            from_interface: "GET:/api/users".to_string(),
            to_interface: "GET:/api/users/{id}".to_string(),
            dependency_type: DependencyType::DataDependency,
            field_mappings: vec![],
            description: Some("User service calls auth service".to_string()),
            strength: 8,
        };
        
        assert_eq!(dependency.from_interface, "GET:/api/users");
        assert_eq!(dependency.to_interface, "GET:/api/users/{id}");
        assert_eq!(dependency.strength, 8);
        
        match dependency.dependency_type {
            DependencyType::DataDependency => assert!(true),
            _ => assert!(false, "Expected DataDependency"),
        }
    }

    #[test]
    fn test_parameter_creation() {
        let param = Parameter {
            name: "user_id".to_string(),
            param_type: "path".to_string(),
            data_type: "string".to_string(),
            required: true,
            description: Some("User identifier".to_string()),
            example: Some("123".to_string()),
            nested_fields: None,
        };
        
        assert_eq!(param.name, "user_id");
        assert_eq!(param.param_type, "path");
        assert!(param.required);
    }

    #[test]
    fn test_response_field_creation() {
        let response_field = ResponseField {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            description: Some("User ID".to_string()),
            example: Some("123".to_string()),
            nested_fields: None,
        };
        
        assert_eq!(response_field.name, "id");
        assert_eq!(response_field.data_type, "integer");
    }

    #[test]
    fn test_field_mapping_creation() {
        let mapping = FieldMapping {
            source_field: "response.data.user_id".to_string(),
            target_field: "parameters.id".to_string(),
            mapping_type: "direct".to_string(),
            transform_rule: None,
        };
        
        assert_eq!(mapping.source_field, "response.data.user_id");
        assert_eq!(mapping.target_field, "parameters.id");
        assert_eq!(mapping.mapping_type, "direct");
    }

    #[test]
    fn test_request_models() {
        let store_request = StoreInterfaceRelationRequest {
            interfaces: vec![],
            dependencies: vec![],
            project_id: "test_project".to_string(),
            version: Some("1.0.0".to_string()),
        };
        
        let query_request = QueryInterfaceRequest {
            query: "user".to_string(),
            project_id: Some("test_project".to_string()),
            max_results: Some(10),
            include_dependencies: Some(true),
        };
        
        assert_eq!(query_request.query, "user");
        assert_eq!(query_request.max_results, Some(10));
        assert_eq!(store_request.project_id, "test_project");
    }

    #[test]
    fn test_dependency_types() {
        let types = vec![
            DependencyType::DataDependency,
            DependencyType::BusinessFlow,
            DependencyType::ParameterPassing,
            DependencyType::AuthDependency,
            DependencyType::ValidationDependency,
            DependencyType::Related,
        ];
        
        assert_eq!(types.len(), 6);
        
        // 测试序列化和反序列化
        for dep_type in types {
            let serialized = serde_json::to_string(&dep_type).unwrap();
            let deserialized: DependencyType = serde_json::from_str(&serialized).unwrap();
            
            match (dep_type, deserialized) {
                (DependencyType::DataDependency, DependencyType::DataDependency) => {},
                (DependencyType::BusinessFlow, DependencyType::BusinessFlow) => {},
                (DependencyType::ParameterPassing, DependencyType::ParameterPassing) => {},
                (DependencyType::AuthDependency, DependencyType::AuthDependency) => {},
                (DependencyType::ValidationDependency, DependencyType::ValidationDependency) => {},
                (DependencyType::Related, DependencyType::Related) => {},
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
    }

    #[test]
    fn test_complex_interface_with_parameters() {
        let interface = ApiInterface {
            path: "/api/users/{id}".to_string(),
            method: "GET".to_string(),
            name: "Get User by ID".to_string(),
            description: Some("Retrieve a specific user by their ID".to_string()),
            parameters: vec![
                Parameter {
                    name: "id".to_string(),
                    param_type: "path".to_string(),
                    data_type: "integer".to_string(),
                    required: true,
                    description: Some("User ID".to_string()),
                    example: Some("123".to_string()),
                    nested_fields: None,
                },
                Parameter {
                    name: "include_profile".to_string(),
                    param_type: "query".to_string(),
                    data_type: "boolean".to_string(),
                    required: false,
                    description: Some("Include user profile data".to_string()),
                    example: Some("true".to_string()),
                    nested_fields: None,
                },
            ],
            responses: vec![
                ResponseField {
                    name: "id".to_string(),
                    data_type: "integer".to_string(),
                    description: Some("User ID".to_string()),
                    example: Some("123".to_string()),
                    nested_fields: None,
                },
                ResponseField {
                    name: "name".to_string(),
                    data_type: "string".to_string(),
                    description: Some("User name".to_string()),
                    example: Some("John Doe".to_string()),
                    nested_fields: None,
                },
            ],
            tags: vec!["user".to_string(), "profile".to_string()],
            domain: Some("user".to_string()),
            deprecated: false,
        };
        
        assert_eq!(interface.parameters.len(), 2);
        assert_eq!(interface.responses.len(), 2);
        assert_eq!(interface.tags.len(), 2);
        
        // 验证路径参数
        let path_param = &interface.parameters[0];
        assert_eq!(path_param.name, "id");
        assert_eq!(path_param.param_type, "path");
        assert!(path_param.required);
        
        // 验证查询参数
        let query_param = &interface.parameters[1];
        assert_eq!(query_param.name, "include_profile");
        assert_eq!(query_param.param_type, "query");
        assert!(!query_param.required);
    }
}