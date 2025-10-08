#[cfg(test)]
mod tests {
    use crate::models::interface_relation::*;

    #[tokio::test]
    async fn test_interface_model_creation() {
        let interface = ApiInterface {
            path: "/api/users".to_string(),
            method: "GET".to_string(),
            summary: Some("Get Users".to_string()),
            description: Some("Get all users".to_string()),
            operation_id: None,
            path_params: vec![],
            query_params: vec![],
            header_params: vec![],
            body_params: vec![],
            request_schema: None,
            response_schema: None,
            tags: vec!["user".to_string()],
            domain: Some("user".to_string()),
            deprecated: false,
            service_description: None,
            embedding: None,
            embedding_model: None,
            embedding_updated_at: None,
        };
        
        assert_eq!(interface.path, "/api/users");
        assert_eq!(interface.method, "GET");
        assert_eq!(interface.summary, Some("Get Users".to_string()));
    }

    #[test]
    fn test_parameter_creation() {
        let param = ApiParameter {
            name: "user_id".to_string(),
            param_type: "path".to_string(),
            required: true,
            description: Some("User identifier".to_string()),
            example: Some("123".to_string()),
            default_value: None,
            enum_values: None,
            format: None,
        };
        
        assert_eq!(param.name, "user_id");
        assert_eq!(param.param_type, "path");
        assert!(param.required);
    }
}