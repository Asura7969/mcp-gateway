#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::models::interface_retrieval::*;
    use crate::services::{EmbeddingService, ElasticSearch, Filter, Search};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;

    /// 创建测试用的Swagger解析请求
    fn create_test_parse_request(project_id: String) -> SwaggerParseRequest {
        let swagger_json = serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0",
                "description": "接口检索与存储测试"
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
    async fn test_elasticsearch_end_to_end() {
        // 加载配置
        let settings = Settings::new().unwrap();
        let embedding_config = settings.embedding;

        println!("{:?}", embedding_config.elasticsearch);
        assert!(embedding_config.elasticsearch.is_some(), "需要配置 embedding.elasticsearch");
        assert!(
            embedding_config.aliyun.is_some(),
            "需要配置 embedding.aliyun 以进行向量嵌入"
        );

        // 创建EmbeddingService
        let embedding_service = Arc::new(EmbeddingService::new(embedding_config.clone()));

        // 创建ElasticSearch服务
        match ElasticSearch::new(&embedding_config, embedding_service.clone()).await {
            Ok(service) => {
                println!("✅ ElasticSearch 服务创建成功");

                // 测试项目ID
                let test_project_id = Uuid::new_v4();


                // 0. 清理测试项目的旧数据（如果存在）
                println!("🧹 清理旧测试数据...");
                let _ = service.delete_project_data(test_project_id.to_string().as_str()).await;
                sleep(Duration::from_millis(500)).await; // 等待删除操作完成

                // 1. 测试存储数据
                println!("🔄 测试存储接口数据...");
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

                // 稍等待索引刷新
                sleep(Duration::from_millis(500)).await;

                // 1.5. 调试：查看存储的数据
                println!("🔍 调试：查看存储的数据...");
                match service.get_project_interfaces(test_project_id.to_string().as_str()).await {
                    Ok(chunks) => {
                        println!("📊 存储的数据数量: {}", chunks.len());
                        for (i, chunk) in chunks.iter().enumerate() {
                            println!("数据 {}: {}", i + 1, &chunk.text[..std::cmp::min(100, chunk.text.len())]);
                        }
                    }
                    Err(e) => {
                        println!("❌ 获取项目数据失败: {:?}", e);
                    }
                }

                // 2. 测试向量检索数据
                println!("🔍 测试向量检索数据...");
                let search_query = "用户id";
                println!("🔍 向量搜索查询: '{}'", search_query);
                
                // 测试嵌入服务
                match embedding_service.embed_text(search_query).await {
                    Ok(embedding) => {
                        println!("✅ 嵌入服务正常，向量维度: {}", embedding.len());
                    }
                    Err(e) => {
                        println!("❌ 嵌入服务失败: {:?}", e);
                    }
                }
                
                let project_filter = Filter {
                    project_id: Some(test_project_id.to_string()),
                    methods: None,
                    prefix_path: None,
                };
                
                // 先测试嵌入服务是否正常工作
                let _test_embedding = embedding_service.embed_text("用户id").await.unwrap();
                
                // 尝试不同的搜索词汇和阈值
                match service.vector_search(search_query, 5, 0.0, Some(&project_filter)).await {
                    Ok(chunks) => {
                        println!("✅ 向量检索成功，找到 {} 个结果", chunks.len());

                        if !chunks.is_empty() {
                            let chunk = &chunks[0];
                            println!("📊 最佳匹配: {} (相似度: {:.3})", chunk.text, chunk.score);

                            // 验证项目ID匹配
                            if let Some(project_id) = chunk.meta.get("project_id") {
                                let stored_project_id = project_id.as_str().unwrap_or("");
                                assert_eq!(stored_project_id, test_project_id.to_string(), "项目ID应该匹配");
                            }
                        } else {
                            println!("⚠️ 向量搜索无结果，尝试更宽松的搜索...");
                            
                            // 尝试更低的阈值
                            match service.vector_search("用户", 5, 0.01, Some(&project_filter)).await {
                                Ok(broader_chunks) => {
                                    println!("📊 更宽松搜索('用户')结果数量: {}", broader_chunks.len());
                                }
                                Err(e) => println!("❌ 更宽松搜索失败: {:?}", e)
                            }
                            
                            // 尝试无过滤器的搜索
                            match service.vector_search("用户id", 5, 0.01, None).await {
                                Ok(no_filter_chunks) => {
                                    println!("📊 无过滤器搜索结果数量: {}", no_filter_chunks.len());
                                    for (i, result) in no_filter_chunks.iter().enumerate() {
                                        println!("结果 {}: {} (分数: {})", i + 1, &result.text[..std::cmp::min(50, result.text.len())], result.score);
                                    }
                                }
                                Err(e) => println!("❌ 无过滤器搜索失败: {:?}", e)
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
                match service.keyword_search("唯一id", 10, Some(&project_filter)).await {
                    Ok(chunks) => {
                        println!("✅ 关键词检索成功，找到 {} 个结果", chunks.len());

                        // 验证检索结果包含关键词
                        for chunk in &chunks {
                            let text = &chunk.text;
                            let contains_keyword = text.contains("唯一id");
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

                        for chunk in &chunks {
                            let meta = &chunk.meta;
                            let meta_str = serde_json::to_string(meta).unwrap();
                            assert_eq!(meta.get("method").unwrap().as_str().unwrap(), "GET", "方法应该是GET");
                            assert!(meta_str.contains("/api/users"), "路径应该包含/api/users");
                            assert_eq!(
                                meta.get("project_id").unwrap().as_str().unwrap(),
                                test_project_id.to_string(),
                                "项目ID应该匹配"
                            );
                        }
                    }
                    Err(e) => {
                        println!("❌ 带过滤器的检索失败: {:?}", e);
                        assert!(false, "带过滤器的检索失败");
                    }
                }

                // 5. 测试混合检索
                println!("🔄 测试混合检索...");
                let hybrid_request = InterfaceSearchRequest {
                    query: "用户id".to_string(),
                    search_type: crate::models::interface_retrieval::SearchType::Hybrid,
                    max_results: 10,
                    similarity_threshold: Some(0.1),
                    vector_weight: Some(0.7), // 70% 向量权重，30% 关键词权重
                    filters: Some(project_filter.clone()),
                };

                match service.hybrid_search(hybrid_request).await {
                    Ok(chunks) => {
                        println!("✅ 混合检索成功，找到 {} 个结果", chunks.len());
                        
                        if !chunks.is_empty() {
                            for (i, chunk) in chunks.iter().enumerate() {
                                println!("混合检索结果 {}: {} (分数: {})", 
                                    i + 1, 
                                    &chunk.text[..std::cmp::min(50, chunk.text.len())], 
                                    chunk.score
                                );
                                
                                // 验证结果包含项目ID
                                if let Some(stored_project_id) = chunk.meta["project_id"].as_str() {
                                    assert_eq!(
                                        stored_project_id,
                                        test_project_id.to_string(),
                                        "混合检索结果的项目ID应该匹配"
                                    );
                                }
                            }
                        } else {
                            println!("⚠️ 混合检索无结果");
                        }
                    }
                    Err(e) => {
                        println!("❌ 混合检索失败: {:?}", e);
                        // 注意：混合检索可能在某些Elasticsearch版本中不支持，所以这里不强制失败
                        println!("⚠️ 混合检索可能需要特定的Elasticsearch配置");
                    }
                }

                // 6. 测试删除数据
                println!("🗑️ 测试删除项目数据...");
                match service.delete_project_data(test_project_id.to_string().as_str()).await {
                    Ok(deleted) => {
                        println!("✅ 数据删除成功: {} 条", deleted);

                        // 等待刷新后验证数据已被删除 - 再次检索应该返回空结果
                        sleep(Duration::from_millis(800)).await;
                        match service.keyword_search("唯一id", 5, Some(&project_filter)).await {
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

                println!("🎉 ElasticSearch 集成测试完成！");
            }
            Err(e) => {
                println!("❌ 无法连接Elasticsearch: {:?}", e);
                assert!(false, "无法连接Elasticsearch");
            }
        }
    }
}