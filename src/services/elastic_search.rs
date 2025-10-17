use crate::config::EmbeddingConfig;
use crate::models::interface_retrieval::*;
use crate::models::swagger::SwaggerSpec;
use crate::services::{merge_content, Chunk, EmbeddingService, Filter, Search};
use crate::utils::generate_api_details;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use elasticsearch::http::transport::Transport;
use elasticsearch::indices::IndicesCreateParts;
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
        let uuid = Uuid::parse_str(source["_id"].as_str().unwrap_or("")).unwrap();
        Self {
            id: uuid,
            text: source["page_content"].to_string(),
            meta: metadata.clone(),
            score,
            embedding: Vec::with_capacity(0),
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
        if create_response.status_code().is_success() {
            info!("Index '{}' created successfully!", INDEX);
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to create index. Response: {:?}",
                create_response
            ))
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
            body.push(
                json!({
                    "page_content": text,
                    "vector": embedding,
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
                    error_count -= items.len();
                    error!("Index errors: {:?}", items);
                }
            }
        }

        Ok((interfaces.len() - error_count) as u32)
    }

    fn build_filter(&self, filters: Option<&Filter>) -> Vec<Value> {
        let mut filter = vec![];
        if let Some(f) = filters {
            if let Some(pid) = &f.project_id {
                filter.push(json!({"terms": {"metadata.project_id": [pid]}}));
            }
            if let Some(methods) = &f.methods {
                filter.push(json!({"terms": {"metadata.method": methods}}));
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
        let filter = self.build_filter(filters);
        if !filter.is_empty() {
            knn.insert("filter".to_string(), Value::Array(filter));
        }
        knn
    }
}

#[async_trait]
impl Search for ElasticSearch {
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

        // 存储接口
        let stored_count = self
            .store_interfaces(&interfaces, &request.project_id)
            .await?;

        info!(
            "Successfully stored {} interfaces for project {}",
            stored_count, request.project_id
        );

        Ok(())
    }

    ///
    /// ```json
    /// {
    ///   "knn": {
    ///       "field": "vector",
    ///       "query_vector": query_embedding,
    ///       "k": max_results,
    ///       "num_candidates": 10000, // 每个分片考虑的候选向量数，影响精度和速度[8](@ref)
    ///       "filter": [
    ///           {"terms": {"metadata.project_id": ["project_123", "project_456"]}},
    ///           {"terms": {"metadata.method": ["GET", "POST"]}},
    ///           {"prefix": {"metadata.path": "/api/v1"}}
    ///       ]
    ///   },
    ///   "fields": ["page_content", "metadata"],
    ///   "_source": false,
    ///   "size": max_results
    /// }
    /// ```
    async fn vector_search(
        &self,
        query: &str,
        max_results: u32,
        _similarity_threshold: f32,
        filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>> {
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
        root.insert(
            "fields".to_string(),
            Value::Array(vec![
                Value::String("page_content".to_string()),
                Value::String("metadata".to_string()),
            ]),
        );
        root.insert("_source".to_string(), Value::Bool(false));
        root.insert("size".to_string(), Value::Number(Number::from(max_results)));

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        extract_response(response_body)
    }

    ///
    /// ```json
    /// {
    ///   "bool": {
    ///       "must": {
    ///           "match": {
    ///               "page_content": query,
    ///           }
    ///       },
    ///       "filter": [
    ///           {"terms": { "metadata.document_id": ["project1", "project2"] }},
    ///           {"terms": { "metadata.method": [ "GET", "POST"] }},
    ///           {"prefix": { "metadata.path": "/api" }}
    ///       ],
    ///       "sort": [{
    ///           "_score": {
    ///               "order": "desc"
    ///           }
    ///       }],
    ///   }
    /// }
    /// ```
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

        bool.insert(
            "sort".to_string(),
            Value::Array(vec![json!({
                "_score": {
                    "order": "desc"
                }
            })]),
        );
        let mut root = serde_json::map::Map::new();
        root.insert("bool".to_string(), Value::Object(bool));
        root.insert("size".to_string(), Value::Number(Number::from(max_results)));

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        extract_response(response_body)
    }

    /// ```json
    /// {
    ///   "query": {
    ///     "bool": {
    ///       "must": [{
    ///           "match": {
    ///             "page_content": {
    ///               "query": "特定关键词",
    ///               "boost": 0.8
    ///             }
    ///           }
    ///         },{
    ///           "knn": {
    ///             "field": "vector",
    ///             "query_vector": [0.12, 0.23, ...], // 1024维查询向量
    ///             "k": 5,
    ///             "num_candidates": 50,
    ///             "boost": 0.2
    ///           }
    ///         }
    ///       ],
    ///       "filter": [ // 在这里添加所有过滤条件
    ///         {"terms": {"metadata.project_id": ["project_123", "project_456"]}},
    ///         {"terms": {"metadata.method": ["GET", "POST"]}},
    ///         {"prefix": {"metadata.path": "/api/v1"}}
    ///       ]
    ///     }
    ///   },
    ///   "size": 10
    /// }
    /// ```
    async fn hybrid_search(&self, request: InterfaceSearchRequest) -> Result<Vec<Chunk>> {
        let mut bool = serde_json::map::Map::new();
        let mut _match = serde_json::map::Map::new();

        let (vector_weight, keyword_weight) = match &request.vector_weight {
            None => (0.0f32, 1f32),
            Some(vector_weight) => (*vector_weight, 1.0 - vector_weight),
        };
        _match.insert(
            "match".to_string(),
            json!({
                "page_content": &request.query,
                "boost": keyword_weight,
            }),
        );

        let query_embedding = self
            .embedding_service
            .embed_text(&request.query)
            .await?
            .into_iter()
            .map(|embedding| embedding.into())
            .collect();

        let knn = self.build_knn(
            query_embedding,
            request.max_results,
            request.filters.as_ref(),
            Some(vector_weight),
        );

        bool.insert(
            "must".to_string(),
            Value::Array(vec![Value::Object(_match), Value::Object(knn)]),
        );
        let filter = self.build_filter(request.filters.as_ref());
        if !filter.is_empty() {
            bool.insert("filter".to_string(), Value::Array(filter));
        }
        let mut root = serde_json::map::Map::new();
        let mut query = serde_json::map::Map::new();
        query.insert("bool".to_string(), Value::Object(bool));
        root.insert("query".to_string(), Value::Object(query));
        root.insert(
            "size".to_string(),
            Value::Number(Number::from(request.max_results)),
        );

        let search_response = self
            .client
            .search(SearchParts::Index(&[INDEX]))
            .body(Value::Object(root))
            .send()
            .await?;
        let response_body = search_response.json::<Value>().await?;

        extract_response(response_body)
    }

    ///
    /// ```json
    /// {
    ///   "bool": {
    ///       "filter": [
    ///           {"terms": { "metadata.document_id": ["project1"] }}
    ///       ],
    ///       "sort": [{
    ///           "_score": {
    ///               "order": "desc"
    ///           }
    ///       }],
    ///   }
    /// }
    /// ```
    async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<Chunk>> {
        let mut bool = serde_json::map::Map::new();
        let filter = Filter {
            project_id: Some(project_id.to_string()),
            prefix_path: None,
            methods: None,
        };
        let filter = self.build_filter(Some(&filter));
        bool.insert("filter".to_string(), Value::Array(filter));

        bool.insert(
            "sort".to_string(),
            Value::Array(vec![json!({
                "_score": {
                    "order": "desc"
                }
            })]),
        );
        let mut root = serde_json::map::Map::new();
        root.insert("bool".to_string(), Value::Object(bool));

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
            .client
            .delete_by_query(DeleteByQueryParts::Index(&[INDEX]))
            .body(json!({
                "query": {
                    "terms": {
                        "metadata.project_id": [project_id]
                    }
                }
            }))
            .send()
            .await?;

        let response_body = response.json::<serde_json::Value>().await?;
        if let Some(deleted_count) = response_body["deleted"].as_u64() {
            Ok(deleted_count)
        } else {
            Err(anyhow!("未能获取删除的文档数量"))
        }
    }
}
