use crate::config::EmbeddingConfig;
use crate::models::interface_retrieval::*;
use crate::models::swagger::SwaggerSpec;
use crate::services::{merge_content, Chunk, EmbeddingService, Filter, Meta, Search};
use crate::utils::generate_api_details;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use elasticsearch::http::transport::Transport;
use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::indices::IndicesRefreshParts;
use elasticsearch::{BulkParts, DeleteByQueryParts, Elasticsearch, SearchParts};
use serde_json::{json, Map, Number, Value};
use std::sync::Arc;
use tracing::log::error;
use tracing::{debug, info};
use uuid::Uuid;

const INDEX: &str = "interface_v2";

impl From<&Value> for Chunk {
    fn from(hit: &Value) -> Self {
        let source = &hit["_source"];
        let score = hit["_score"].as_f64().unwrap_or(0.0);
        let metadata = &source["metadata"];
        // 修复：_id 应该来自命中顶层而不是 _source
        let uuid_str = hit["_id"].as_str().unwrap_or("");
        let uuid = Uuid::parse_str(uuid_str).unwrap_or_else(|_| Uuid::new_v4());

        // 从Elasticsearch的vector字段读取嵌入向量
        let embedding: Vec<f32> = source["vector"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        let api_content = match source["api_content"].as_str() {
            None => None,
            Some(api_content_str) => {
                let mut api_interface =
                    serde_json::from_str::<ApiInterface>(api_content_str).unwrap();
                // 如果嵌入向量全为零，则设置为None，否则设置为Some
                api_interface.embedding = if embedding.iter().all(|&x| x == 0.0) {
                    None
                } else {
                    Some(embedding.clone())
                };
                Some(api_interface)
            }
        };

        Self {
            id: uuid,
            // 修复：避免使用 to_string() 导致带引号的 JSON 字符串
            text: source["page_content"].as_str().unwrap_or("").to_string(),
            meta: metadata.clone(),
            score,
            embedding,
            api_content,
            created_at: None,
            updated_at: None,
        }
    }
}

fn extract_response(response_body: Value) -> Result<Vec<Chunk>> {
    if let Some(hits) = response_body["hits"]["hits"].as_array() {
        Ok(hits.iter().map(Chunk::from).collect())
    } else {
        Ok(Vec::with_capacity(0))
    }
}

/// Elastic 搜索服务
pub struct ElasticSearch {
    client: Elasticsearch,
    embedding_service: Arc<EmbeddingService>,
}

impl ElasticSearch {
    /// 创建新的服务实例
    pub async fn new(
        config: &EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self> {
        let elastic_config = config
            .elasticsearch
            .as_ref()
            .ok_or_else(|| anyhow!("Elasticsearch configuration not found"))?;
        let url = format!(
            r#"http://{}:{}@{}:{}"#,
            elastic_config.user, elastic_config.password, elastic_config.host, elastic_config.port
        );

        let transport = Transport::single_node(&url)?;
        let client = Elasticsearch::new(transport);
        if let Err(_) = client.ping().send().await {
            return Err(anyhow!("Elasticsearch connection error"));
        }

        let service = Self {
            client,
            embedding_service,
        };
        service.init_schema().await?;
        Ok(service)
    }

    /// 初始化数据库schema
    async fn init_schema(&self) -> Result<()> {
        let create_response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(INDEX))
            .body(json!({
                "mappings": {
                    "properties": {
                        "page_content": {
                            "type": "text",
                            "analyzer": "ik_max_word",
                            "search_analyzer": "ik_smart"
                        },
                        "api_content": {
                            "type": "text",
                        },
                        "vector": {
                            "type": "dense_vector",
                            "dims": 1024,
                            "index": true,
                            "similarity": "cosine",
                        },
                        "metadata": {
                            "type": "object",
                                "properties": {
                                    "project_id": {"type": "keyword"},
                                    "path": {"type": "keyword"},
                                    "method": {"type": "keyword"},
                                },
                        }
                    }
                }
            }))
            .send()
            .await?;
        let status = create_response.status_code();
        if status.is_success() || status.as_u16() == 400 {
            info!("Index '{}' ready!", INDEX);
            Ok(())
        } else {
            Err(anyhow!("Failed to create index. Status: {:?}", status))
        }
    }

    /// 存储接口到数据库
    async fn store_interfaces(&self, interfaces: &[ApiInterface], project_id: &str) -> Result<u32> {
        let mut body: Vec<String> = Vec::new();

        for interface in interfaces {
            body.push(
                json!({
                    "index": {
                        "_index": INDEX,
                        "_id": Uuid::new_v4().to_string().as_str()
                    }
                })
                .to_string(),
            );

            let text = merge_content(interface);
            let embedding = self.embedding_service.embed_text(&text).await?;
            let api_content = serde_json::to_string::<ApiInterface>(interface).unwrap();

            body.push(
                json!({
                    "page_content": text,
                    "vector": embedding,
                    "api_content": api_content,
                    "metadata": {
                        "project_id": project_id,
                        "path": interface.path,
                        "method": interface.method
                    }
                })
                .to_string(),
            );
        }

        let response = self
            .client
            .bulk(BulkParts::Index(INDEX))
            .body(body)
            .send()
            .await?;
        let response_body = response.json::<Value>().await?;

        debug!("Response body: {:?}", response_body);

        let mut error_count = 0;
        if let Some(errors) = response_body["errors"].as_bool() {
            if errors {
                if let Some(items) = response_body["items"].as_array() {
                    error_count += items.len();
                    error!("Index errors: {:?}", items);
                }
            }
        }

        // 刷新索引以确保数据立即可搜索
        let _refresh_response = self
            .client
            .indices()
            .refresh(IndicesRefreshParts::Index(&[INDEX]))
            .send()
            .await?;

        Ok((interfaces.len() - error_count) as u32)
    }

    async fn store_interfaces_without_embeddings(
        &self,
        interfaces: &[ApiInterface],
        project_id: &str,
    ) -> Result<u32> {
        let mut body: Vec<String> = Vec::new();

        for interface in interfaces {
            body.push(
                json!({
                    "index": {
                        "_index": INDEX,
                        "_id": Uuid::new_v4().to_string().as_str()
                    }
                })
                .to_string(),
            );

            let text = merge_content(interface);
            // 使用零向量作为占位符
            let embedding: Vec<f32> = vec![0.0; 1024];
            let api_content = serde_json::to_string::<ApiInterface>(interface).unwrap();

            body.push(
                json!({
                    "page_content": text,
                    "vector": embedding,
                    "api_content": api_content,
                    "metadata": {
                        "project_id": project_id,
                        "path": interface.path,
                        "method": interface.method
                    }
                })
                .to_string(),
            );
        }

        let response = self
            .client
            .bulk(BulkParts::Index(INDEX))
            .body(body)
            .send()
            .await?;
        let response_body = response.json::<Value>().await?;

        debug!("Response body: {:?}", response_body);

        let mut error_count = 0;
        if let Some(errors) = response_body["errors"].as_bool() {
            if errors {
                if let Some(items) = response_body["items"].as_array() {
                    error_count += items.len();
                    error!("Index errors: {:?}", items);
                }
            }
        }

        // 刷新索引以确保数据立即可搜索
        let _refresh_response = self
            .client
            .indices()
            .refresh(IndicesRefreshParts::Index(&[INDEX]))
            .send()
            .await?;

        Ok((interfaces.len() - error_count) as u32)
    }

    fn build_filter(&self, filters: Option<&Filter>) -> Vec<Value> {
        let mut filter = vec![];
        if let Some(f) = filters {
            if let Some(pid) = &f.project_id {
                filter.push(json!({"term": {"metadata.project_id": pid}}));
            }
            if let Some(methods) = &f.methods {
                if !methods.is_empty() {
                    filter.push(json!({"terms": {"metadata.method": methods}}));
                }
            }
            if let Some(prefix_path) = &f.prefix_path {
                filter.push(json!({"prefix": {"metadata.path": prefix_path}}));
            }
        }
        filter
    }

    fn build_knn(
        &self,
        query_vector: Vec<Value>,
        max_results: u32,
        filters: Option<&Filter>,
        weight: Option<f32>,
    ) -> Map<String, Value> {
        let mut knn = serde_json::map::Map::new();
        knn.insert("field".to_string(), Value::String("vector".to_string()));
        knn.insert("query_vector".to_string(), Value::Array(query_vector));
        knn.insert("k".to_string(), Value::Number(Number::from(max_results)));
        knn.insert(
            "num_candidates".to_string(),
            Value::Number(Number::from(10000)),
        );
        if let Some(w) = weight {
            knn.insert("boost".to_string(), json!(w));
        }
        let filter_clauses = self.build_filter(filters);
        if !filter_clauses.is_empty() {
            // 对于KNN查询，过滤器应该是一个完整的bool查询对象
            let mut bool_obj = serde_json::map::Map::new();
            bool_obj.insert("must".to_string(), Value::Array(filter_clauses));

            let mut filter_obj = serde_json::map::Map::new();
            filter_obj.insert("bool".to_string(), Value::Object(bool_obj));

            knn.insert("filter".to_string(), Value::Object(filter_obj));
        }
        knn
    }

    async fn delete(&self, body: Value) -> Result<Value> {
        let response = self
            .client
            .delete_by_query(DeleteByQueryParts::Index(&[INDEX]))
            .body(body)
            .send()
            .await?;

        let response_body = response.json::<Value>().await?;

        // 刷新索引以确保删除操作立即生效
        let _refresh_response = self
            .client
            .indices()
            .refresh(IndicesRefreshParts::Index(&[INDEX]))
            .send()
            .await?;
        Ok(response_body)
    }
}

#[async_trait]
impl Search for ElasticSearch {
    async fn store_interface(&self, interface: ApiInterface, project_id: String) -> Result<()> {
        let _ = self
            .store_interfaces(&[interface], project_id.as_str())
            .await?;
        Ok(())
    }

    async fn parse_and_store_swagger(&self, request: SwaggerParseRequest) -> Result<()> {
        info!("Parsing Swagger for project: {}", request.project_id);

        // 解析Swagger JSON
        let swagger_spec: SwaggerSpec = serde_json::from_value(request.swagger_json)?;
        let api_details = generate_api_details(&swagger_spec)?;

        info!("Found {} interfaces in Swagger", api_details.len());

        // 转换为ApiInterface
        let interfaces: Vec<ApiInterface> = api_details
            .into_iter()
            .map(|detail| {
                let mut interface = ApiInterface::from(detail);
                interface.service_description = swagger_spec.info.description.clone();
                interface.tags = vec![swagger_spec.info.title.clone()];
                interface
            })
            .collect();

        // 根据generate_embeddings参数决定是否生成嵌入向量
        let stored_count = if request.generate_embeddings.unwrap_or(false) {
            self.store_interfaces(&interfaces, &request.project_id)
                .await?
        } else {
            self.store_interfaces_without_embeddings(&interfaces, &request.project_id)
                .await?
        };

        info!(
            "Successfully stored {} interfaces for project {}",
            stored_count, request.project_id
        );

        Ok(())
    }

    async fn vector_search(
        &self,
        query: &str,
        max_results: u32,
        similarity_threshold: f32,
        filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>> {
        info!("filter: {:?}", filters);
        // 获取查询向量
        let query_embedding = self
            .embedding_service
            .embed_text(query)
            .await?
            .into_iter()
            .map(|embedding| embedding.into())
            .collect();

        let mut root = serde_json::map::Map::new();

        let knn = self.build_knn(query_embedding, max_results, filters, None);
        root.insert("knn".to_string(), Value::Object(knn));
        // 返回完整 _source，便于解析 text 与 metadata
        root.insert("_source".to_string(), Value::Bool(true));
        root.insert("size".to_string(), Value::Number(Number::from(max_results)));

        let query_json = serde_json::to_string_pretty(&Value::Object(root.clone())).unwrap();
        info!("🔍 Vector search query: {}", query_json);

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        let mut results = extract_response(response_body)?;

        // 应用相似度阈值过滤
        if similarity_threshold > 0.0 {
            results.retain(|chunk| chunk.score >= similarity_threshold as f64);
        }

        Ok(results)
    }

    async fn keyword_search(
        &self,
        query: &str,
        max_results: u32,
        filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>> {
        let mut bool = serde_json::map::Map::new();
        let mut must = serde_json::map::Map::new();
        must.insert(
            "match".to_string(),
            json!({
                "page_content": query,
            }),
        );

        bool.insert("must".to_string(), Value::Object(must));
        let filter = self.build_filter(filters);
        if !filter.is_empty() {
            bool.insert("filter".to_string(), Value::Array(filter));
        }

        let mut root = serde_json::map::Map::new();
        // 修复：Elasticsearch 搜索必须包含 query 包裹 bool
        let mut query_obj = serde_json::map::Map::new();
        query_obj.insert("bool".to_string(), Value::Object(bool));
        root.insert("query".to_string(), Value::Object(query_obj));
        root.insert("size".to_string(), Value::Number(Number::from(max_results)));
        root.insert(
            "sort".to_string(),
            Value::Array(vec![json!({
                "_score": {
                    "order": "desc"
                }
            })]),
        );

        let query_json = serde_json::to_string_pretty(&Value::Object(root.clone())).unwrap();
        info!("🔍 Keyword search query: {}", query_json);

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        extract_response(response_body)
    }

    async fn hybrid_search(&self, request: InterfaceSearchRequest) -> Result<Vec<Chunk>> {
        let (vector_weight, keyword_weight) = match request.search_type {
            SearchType::Vector => (1.0f32, 0.0f32),
            SearchType::Keyword => (0.0f32, 1.0f32),
            SearchType::Hybrid => {
                match &request.vector_weight {
                    None => (0.5f32, 0.5f32), // 默认权重相等
                    Some(vector_weight) => (*vector_weight, 1.0 - vector_weight),
                }
            }
        };

        let max_results = request.max_results;

        // 分别执行向量搜索和关键词搜索
        let vector_results = self
            .vector_search(
                &request.query,
                max_results,
                0.0, // 不在这里应用阈值，稍后统一处理
                request.filters.as_ref(),
            )
            .await?;

        let keyword_results = self
            .keyword_search(&request.query, max_results, request.filters.as_ref())
            .await?;

        // 手动合并结果并应用权重
        let mut combined_results: std::collections::HashMap<String, Chunk> =
            std::collections::HashMap::new();

        // 添加向量搜索结果
        for mut chunk in vector_results {
            chunk.score = chunk.score * vector_weight as f64;
            combined_results.insert(chunk.id.to_string(), chunk);
        }

        // 添加关键词搜索结果，如果已存在则合并分数
        for mut chunk in keyword_results {
            chunk.score = chunk.score * keyword_weight as f64;
            if let Some(existing) = combined_results.get_mut(&chunk.id.to_string()) {
                existing.score += chunk.score;
            } else {
                combined_results.insert(chunk.id.to_string(), chunk);
            }
        }

        // 转换为向量并按分数排序
        let mut results: Vec<Chunk> = combined_results.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制结果数量
        results.truncate(max_results as usize);

        // 应用相似度阈值过滤
        if let Some(threshold) = request.similarity_threshold {
            if threshold > 0.0 {
                results.retain(|chunk| chunk.score >= threshold as f64);
            }
        }

        println!("🔍 混合检索成功，找到 {} 个结果", results.len());
        for (i, chunk) in results.iter().enumerate() {
            println!("  结果 {}: ID={}, 分数={:.6}", i + 1, chunk.id, chunk.score);
        }

        Ok(results)
    }

    async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<Chunk>> {
        let mut bool = serde_json::map::Map::new();

        // 添加match_all查询
        bool.insert("must".to_string(), json!([{"match_all": {}}]));

        let filter = Filter {
            project_id: Some(project_id.to_string()),
            prefix_path: None,
            methods: None,
        };
        let filter = self.build_filter(Some(&filter));
        bool.insert("filter".to_string(), Value::Array(filter));

        let mut root = serde_json::map::Map::new();
        let mut query_obj = serde_json::map::Map::new();
        query_obj.insert("bool".to_string(), Value::Object(bool));
        root.insert("query".to_string(), Value::Object(query_obj));
        root.insert("size".to_string(), Value::Number(Number::from(100))); // 设置返回数量

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        extract_response(response_body)
    }

    async fn delete_project_data(&self, project_id: &str) -> Result<u64> {
        let response = self
            .delete(json!({
                "query": {
                    "term": {
                        "metadata.project_id": project_id
                    }
                }
            }))
            .await?;

        if let Some(deleted_count) = response["deleted"].as_u64() {
            Ok(deleted_count)
        } else {
            Err(anyhow!("未能获取删除的文档数量"))
        }
    }

    async fn delete_by_meta(&self, meta: Meta) -> Result<()> {
        if meta.any_empty() {
            return Err(anyhow!("Meta is empty"));
        }

        let response = self
            .delete(json!({
                "query": {
                    "term": {
                        "metadata.project_id": meta.project_id,
                        "metadata.path": meta.path,
                        "metadata.method": meta.method,
                    }
                }
            }))
            .await?;

        if let Some(_) = response["deleted"].as_u64() {
            Ok(())
        } else {
            Err(anyhow!("未能获取删除的文档数量"))
        }
    }
}
