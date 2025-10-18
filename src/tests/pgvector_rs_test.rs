#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::models::interface_retrieval::*;
    use crate::services::{EmbeddingService, Filter, PgvectorRsSearch, Search};
    use std::sync::Arc;
    use uuid::Uuid;

    /// 创建测试用的Swagger解析请求
    fn create_test_parse_request(project_id: String) -> SwaggerParseRequest {
        let swagger_json = serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/api/users/{id}": {
                    "get": {
                        "summary": "获取用户id",
                        "description": "通过唯一id检索指定用户",
                        "operationId": "getUserById",
                        "tags": ["users", "profile"],
                        "parameters": [
                            {
                                "name": "id",
                                "in": "path",
                                "required": true,
                                "schema": {"type": "integer", "format": "int64"},
                                "description": "User ID",
                                "example": 123
                            },
                            {
                                "name": "include_profile",
                                "in": "query",
                                "required": false,
                                "schema": {"type": "boolean", "default": false},
                                "description": "Include user profile information",
                                "example": true
                            },
                            {
                                "name": "Authorization",
                                "in": "header",
                                "required": true,
                                "schema": {"type": "string"},
                                "description": "Bearer token for authentication",
                                "example": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
                            }
                        ],
                        "responses": {
                            "200": {
                                "description": "User found",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "object",
                                            "properties": {
                                                "id": {"type": "integer"},
                                                "name": {"type": "string"},
                                                "email": {"type": "string"}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        SwaggerParseRequest {
            project_id,
            swagger_json,
            version: Some("1.0.0".to_string()),
            generate_embeddings: Some(true),
        }
    }

    #[tokio::test]
    async fn test_pgvector_rs_service_creation() {
        let settings = Settings::new().unwrap();
        let embedding_config = settings.embedding;

        println!("{:?}", embedding_config.pgvectorrs);
        assert!(embedding_config.pgvectorrs.is_some());

        // 创建EmbeddingService
        let embedding_service = Arc::new(EmbeddingService::new(embedding_config.clone()));

        match PgvectorRsSearch::new(&embedding_config, embedding_service).await {
            Ok(service) => {
                println!("✅ PgvectorRsService 创建成功");

                // 测试项目ID
                let test_project_id = Uuid::new_v4();

                // 1. 测试存储数据
                println!("🔄 测试存储接口数据...");

                // 创建SwaggerParseRequest来测试存储
                let swagger_request = create_test_parse_request(test_project_id.to_string());
                match service.parse_and_store_swagger(swagger_request).await {
                    Ok(_) => {
                        println!("✅ 接口数据存储成功");
                    }
                    Err(e) => {
                        println!("❌ 接口数据存储失败: {:?}", e);
                        assert!(false, "接口数据存储失败");
                    }
                }

                // 2. 测试向量检索数据
                println!("🔍 测试向量检索数据...");
                let search_query = "用户id";
                match service.vector_search(search_query, 5, 0.5, None).await {
                    Ok(chunks) => {
                        println!("✅ 向量检索成功，找到 {} 个结果", chunks.len());

                        // 验证检索结果
                        if !chunks.is_empty() {
                            let chunk = &chunks[0];
                            println!("📊 最佳匹配: {} (相似度: {:.3})", chunk.meta, chunk.score);

                            // 验证项目ID匹配
                            if let Some(project_id) = chunk.meta.get("project_id") {
                                let stored_project_id = project_id.as_str().unwrap_or("");
                                assert_eq!(
                                    stored_project_id,
                                    test_project_id.to_string(),
                                    "项目ID应该匹配"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ 向量检索失败: {:?}", e);
                        assert!(false, "向量检索失败");
                    }
                }

                // 3. 测试关键词检索
                println!("🔍 测试关键词检索数据...");
                match service.keyword_search("user", 10, None).await {
                    Ok(chunks) => {
                        println!("✅ 关键词检索成功，找到 {} 个结果", chunks.len());

                        // 验证检索结果包含关键词
                        for chunk in &chunks {
                            let text = &chunk.text;
                            let contains_keyword = text.to_lowercase().contains("唯一id");
                            assert!(contains_keyword, "检索结果应该包含关键词 '唯一id'");
                        }
                    }
                    Err(e) => {
                        println!("❌ 关键词检索失败: {:?}", e);
                        assert!(false, "关键词检索失败");
                    }
                }

                // 4. 测试带过滤器的检索
                println!("🔍 测试带过滤器的检索...");
                let filters = Filter {
                    methods: Some(vec!["GET".to_string()]),
                    project_id: Some(test_project_id.to_string()),
                    prefix_path: Some("/api/users".to_string()),
                };

                match service
                    .vector_search("retrieve user information", 5, 0.3, Some(&filters))
                    .await
                {
                    Ok(chunks) => {
                        println!("✅ 带过滤器的检索成功，找到 {} 个结果", chunks.len());

                        // 验证过滤器效果
                        for chunk in &chunks {
                            let meta = &chunk.meta;
                            let meta_str = serde_json::to_string(meta).unwrap();
                            assert_eq!(
                                meta.get("method").unwrap().as_str().unwrap(),
                                "GET",
                                "方法应该是GET"
                            );
                            assert!(meta_str.contains("/api/users"), "路径应该包含/api/users");
                        }
                    }
                    Err(e) => {
                        println!("❌ 带过滤器的检索失败: {:?}", e);
                        assert!(false, "带过滤器的检索失败");
                    }
                }

                // 5. 测试删除数据
                println!("🗑️ 测试删除项目数据...");
                match service
                    .delete_project_data(test_project_id.to_string().as_str())
                    .await
                {
                    Ok(message) => {
                        println!("✅ 数据删除成功: {}", message);

                        // 验证数据已被删除 - 再次检索应该返回空结果
                        match service.vector_search(search_query, 5, 0.5, None).await {
                            Ok(results) => {
                                assert!(results.is_empty(), "删除后检索应该返回空结果");
                                println!("✅ 验证删除成功：检索返回空结果");
                            }
                            Err(e) => {
                                println!("❌ 验证删除失败: {:?}", e);
                                assert!(false, "验证删除失败");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ 数据删除失败: {:?}", e);
                        assert!(false, "数据删除失败");
                    }
                }

                println!("🎉 所有测试完成！");
            }
            Err(e) => {
                println!("❌ 无法连接pgvectorrs: {:?}", e);
                assert!(false, "无法连接pgvectorrs");
            }
        }
    }
}
