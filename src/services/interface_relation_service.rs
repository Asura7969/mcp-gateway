use crate::models::interface_relation::*;
use crate::models::swagger::SwaggerSpec;
use crate::services::EmbeddingService;
use crate::utils::generate_api_details;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::sql::Datetime;
use surrealdb::Surreal;

/// 解析Swagger请求
#[derive(Debug, Clone)]
pub struct ParseSwaggerRequest {
    pub project_id: String,
    pub swagger_json: Value,
}

/// 带有元数据的接口结构（用于数据库存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceWithMeta {
    pub id: Option<String>,
    pub project_id: String,
    pub path: String,
    pub method: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub path_params: Vec<ApiParameter>,
    pub query_params: Vec<ApiParameter>,
    pub header_params: Vec<ApiParameter>,
    pub body_params: Vec<ApiParameter>,
    pub request_schema: Option<String>,
    pub response_schema: Option<String>,
    pub tags: Vec<String>,
    pub domain: Option<String>,
    pub deprecated: bool,
    pub service_description: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub embedding_model: Option<String>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// 接口关系服务 - 重新设计用于swagger解析和向量搜索
pub struct InterfaceRelationService {
    db: Surreal<Db>,
    embedding_service: Arc<EmbeddingService>,
}

impl InterfaceRelationService {
    /// 创建新的服务实例
    pub async fn new(embedding_service: Arc<EmbeddingService>) -> Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("interface_relation").use_db("main").await?;

        let service = Self {
            db,
            embedding_service,
        };
        service.init_schema().await?;
        Ok(service)
    }

    /// 初始化数据库schema
    async fn init_schema(&self) -> Result<()> {
        // 创建接口表
        self.db.query("DEFINE TABLE interface SCHEMAFULL").await?;

        // 定义接口表字段 - 基于新的ApiInterface结构
        self.db
            .query("DEFINE FIELD path ON TABLE interface TYPE string")
            .await?;
        self.db
            .query("DEFINE FIELD method ON TABLE interface TYPE string")
            .await?;
        self.db
            .query("DEFINE FIELD summary ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD description ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD operation_id ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD path_params ON TABLE interface TYPE array")
            .await?;
        self.db
            .query("DEFINE FIELD query_params ON TABLE interface TYPE array")
            .await?;
        self.db
            .query("DEFINE FIELD header_params ON TABLE interface TYPE array")
            .await?;
        self.db
            .query("DEFINE FIELD body_params ON TABLE interface TYPE array")
            .await?;
        self.db
            .query("DEFINE FIELD request_schema ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD response_schema ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD tags ON TABLE interface TYPE array")
            .await?;
        self.db
            .query("DEFINE FIELD domain ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD deprecated ON TABLE interface TYPE bool")
            .await?;
        self.db
            .query("DEFINE FIELD service_description ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD embedding ON TABLE interface TYPE option<array>")
            .await?;
        self.db
            .query("DEFINE FIELD embedding_model ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD embedding_updated_at ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD project_id ON TABLE interface TYPE string")
            .await?;
        self.db
            .query("DEFINE FIELD version ON TABLE interface TYPE option<string>")
            .await?;
        self.db
            .query("DEFINE FIELD created_at ON TABLE interface TYPE datetime")
            .await?;
        self.db
            .query("DEFINE FIELD updated_at ON TABLE interface TYPE datetime")
            .await?;

        // 创建索引以提高搜索性能
        self.db
            .query("DEFINE INDEX idx_project_id ON TABLE interface COLUMNS project_id")
            .await?;
        self.db
            .query("DEFINE INDEX idx_path_method ON TABLE interface COLUMNS path, method")
            .await?;
        self.db
            .query("DEFINE INDEX idx_tags ON TABLE interface COLUMNS tags")
            .await?;

        Ok(())
    }

    /// 解析Swagger JSON并存储接口信息
    pub async fn parse_and_store_swagger(&self, request: ParseSwaggerRequest) -> Result<()> {
        // 解析 Swagger JSON
        let swagger_spec: SwaggerSpec = serde_json::from_value(request.swagger_json)?;

        // 直接生成 API 详情，避免依赖 EndpointService
        let api_details = generate_api_details(&swagger_spec)?;

        // 转换为 ApiInterface 并存储
        let interfaces: Vec<ApiInterface> = api_details
            .into_iter()
            .map(|detail| {
                let mut interface = ApiInterface::from(detail);
                // 设置服务描述和标签
                interface.service_description = swagger_spec.info.description.clone();
                interface.tags = vec![swagger_spec.info.title.clone()];
                interface
            })
            .collect();

        self.store_interfaces(&interfaces, &request.project_id, None)
            .await?;

        Ok(())
    }

    /// 存储接口到数据库
    async fn store_interfaces(
        &self,
        interfaces: &[ApiInterface],
        project_id: &str,
        _version: Option<&str>,
    ) -> Result<u32> {
        let mut stored_count = 0;

        for interface in interfaces {
            let now = Datetime::from(Utc::now());
            let interface_with_meta = InterfaceWithMeta {
                id: None,
                project_id: project_id.to_string(),
                path: interface.path.clone(),
                method: interface.method.clone(),
                summary: interface.summary.clone(),
                description: interface.description.clone(),
                operation_id: interface.operation_id.clone(),
                path_params: interface.path_params.clone(),
                query_params: interface.query_params.clone(),
                header_params: interface.header_params.clone(),
                body_params: interface.body_params.clone(),
                request_schema: interface.request_schema.clone(),
                response_schema: interface.response_schema.clone(),
                tags: interface.tags.clone(),
                domain: interface.domain.clone(),
                deprecated: interface.deprecated,
                service_description: interface.service_description.clone(),
                embedding: interface.embedding.clone(),
                embedding_model: interface.embedding_model.clone(),
                created_at: now.clone(),
                updated_at: now,
            };

            match self
                .db
                .create::<Option<InterfaceWithMeta>>("interface")
                .content(interface_with_meta)
                .await
            {
                Ok(_) => stored_count += 1,
                Err(e) => {
                    tracing::error!("Failed to store interface {}: {}", interface.path, e);
                }
            }
        }

        Ok(stored_count)
    }

    /// 搜索接口 - 支持关键词和向量搜索
    pub async fn search_interfaces(
        &self,
        request: InterfaceSearchRequest,
    ) -> Result<InterfaceSearchResponse> {
        let start_time = Instant::now();
        let max_results = request.max_results.unwrap_or(10);
        let enable_vector_search = request.enable_vector_search.unwrap_or(false);
        let enable_keyword_search = request.enable_keyword_search.unwrap_or(true);
        let vector_weight = request.vector_search_weight.unwrap_or(0.5);
        let similarity_threshold = request.similarity_threshold.unwrap_or(0.7);

        let mut interfaces = match (enable_vector_search, enable_keyword_search) {
            (true, true) => {
                // 混合搜索：关键词 + 向量
                self.hybrid_search(
                    &request.query,
                    max_results,
                    vector_weight,
                    similarity_threshold,
                )
                .await?
            }
            (true, false) => {
                // 纯向量搜索
                self.search_interfaces_by_vector(&request.query, max_results, similarity_threshold)
                    .await?
            }
            (false, true) => {
                // 纯关键词搜索
                self.search_interfaces_by_keywords(&request.query, max_results)
                    .await?
            }
            (false, false) => {
                // 两种搜索都禁用，返回空结果
                Vec::new()
            }
        };

        // 应用过滤器
        if let Some(filters) = &request.filters {
            interfaces = self.apply_search_filters(interfaces, filters, &request.project_id);
        }

        // 限制结果数量
        interfaces.truncate(max_results as usize);

        let search_mode = match (enable_vector_search, enable_keyword_search) {
            (true, true) => "hybrid",
            (true, false) => "vector",
            (false, true) => "keyword",
            (false, false) => "none",
        }
        .to_string();

        let total_count = interfaces.len() as u32;

        Ok(InterfaceSearchResponse {
            interfaces,
            query_time_ms: start_time.elapsed().as_millis() as u64,
            total_count,
            search_mode,
        })
    }

    /// 基于关键词搜索接口
    async fn search_interfaces_by_keywords(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<InterfaceWithScore>> {
        let _keywords: Vec<&str> = query.split_whitespace().collect();

        // 使用SurrealDB的全文搜索功能
        let search_query = format!(
            "SELECT * FROM interface WHERE 
             string::lowercase(summary) CONTAINS string::lowercase('{}') OR
             string::lowercase(description) CONTAINS string::lowercase('{}') OR
             string::lowercase(service_description) CONTAINS string::lowercase('{}') OR
             string::lowercase(path) CONTAINS string::lowercase('{}')",
            query, query, query, query
        );

        let interfaces: Vec<InterfaceRecord> = self.db.query(&search_query).await?.take(0)?;

        let mut results = Vec::new();
        for record in interfaces {
            let score = self.calculate_match_score(&record.interface, query);
            let match_reason = self.get_match_reason(&record.interface, query);

            results.push(InterfaceWithScore {
                interface: record.interface,
                score,
                match_reason,
                similarity_score: None,
                search_type: "keyword".to_string(),
            });
        }

        // 按分数排序并限制结果数量
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results as usize);

        Ok(results)
    }

    /// 基于向量相似度搜索接口
    async fn search_interfaces_by_vector(
        &self,
        query: &str,
        max_results: u32,
        similarity_threshold: f32,
    ) -> Result<Vec<InterfaceWithScore>> {
        // 1. 生成查询文本的向量
        let query_embedding = self.embedding_service.embed_text(query).await?;

        // 2. 获取所有有向量嵌入的接口
        let interfaces_with_embeddings: Vec<InterfaceRecord> = self
            .db
            .query("SELECT * FROM interface WHERE embedding IS NOT NULL")
            .await?
            .take(0)?;

        let mut results = Vec::new();

        // 3. 计算相似度并筛选
        for record in interfaces_with_embeddings {
            if let Some(embedding) = &record.interface.embedding {
                let similarity = self.calculate_cosine_similarity(&query_embedding, embedding);

                if similarity >= similarity_threshold {
                    let match_reason = format!("向量相似度: {:.3}", similarity);

                    results.push(InterfaceWithScore {
                        interface: record.interface,
                        score: similarity as f64,
                        match_reason,
                        similarity_score: Some(similarity),
                        search_type: "vector".to_string(),
                    });
                }
            }
        }

        // 4. 按相似度排序并限制结果数量
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results as usize);

        Ok(results)
    }

    /// 混合搜索：关键词 + 向量
    async fn hybrid_search(
        &self,
        query: &str,
        max_results: u32,
        vector_weight: f32,
        similarity_threshold: f32,
    ) -> Result<Vec<InterfaceWithScore>> {
        let keyword_results = self
            .search_interfaces_by_keywords(query, max_results * 2)
            .await?;
        let vector_results = self
            .search_interfaces_by_vector(query, max_results * 2, similarity_threshold)
            .await?;

        let mut combined_results = HashMap::new();

        // 合并关键词搜索结果
        for result in keyword_results {
            let key = format!("{}:{}", result.interface.path, result.interface.method);
            combined_results.insert(key, result);
        }

        // 合并向量搜索结果，调整评分
        for mut result in vector_results {
            let key = format!("{}:{}", result.interface.path, result.interface.method);

            if let Some(existing) = combined_results.get_mut(&key) {
                // 混合评分：关键词权重 + 向量权重
                existing.score = existing.score * (1.0 - vector_weight as f64)
                    + result.score * vector_weight as f64;
                existing.search_type = "hybrid".to_string();
                existing.similarity_score = result.similarity_score;
            } else {
                result.search_type = "vector".to_string();
                combined_results.insert(key, result);
            }
        }

        let mut final_results: Vec<InterfaceWithScore> = combined_results.into_values().collect();
        final_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        final_results.truncate(max_results as usize);

        Ok(final_results)
    }

    /// 计算余弦相似度
    fn calculate_cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// 计算匹配分数
    pub fn calculate_match_score(&self, interface: &ApiInterface, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let mut score = 0.0f64;

        // 检查摘要匹配
        if let Some(summary) = &interface.summary {
            if summary.to_lowercase().contains(&query_lower) {
                score += 0.4;
            }
        }

        // 检查描述匹配
        if let Some(description) = &interface.description {
            if description.to_lowercase().contains(&query_lower) {
                score += 0.3;
            }
        }

        // 检查服务描述匹配
        if let Some(service_desc) = &interface.service_description {
            if service_desc.to_lowercase().contains(&query_lower) {
                score += 0.2;
            }
        }

        // 检查路径匹配
        if interface.path.to_lowercase().contains(&query_lower) {
            score += 0.2;
        }

        // 检查标签匹配
        for tag in &interface.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 0.1;
            }
        }

        score.min(1.0)
    }

    /// 获取匹配原因
    fn get_match_reason(&self, interface: &ApiInterface, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let mut reasons = Vec::new();

        if interface.path.to_lowercase().contains(&query_lower) {
            reasons.push("路径匹配");
        }

        if let Some(summary) = &interface.summary {
            if summary.to_lowercase().contains(&query_lower) {
                reasons.push("摘要匹配");
            }
        }

        if let Some(description) = &interface.description {
            if description.to_lowercase().contains(&query_lower) {
                reasons.push("描述匹配");
            }
        }

        if let Some(service_desc) = &interface.service_description {
            if service_desc.to_lowercase().contains(&query_lower) {
                reasons.push("服务描述匹配");
            }
        }

        for tag in &interface.tags {
            if tag.to_lowercase().contains(&query_lower) {
                reasons.push("标签匹配");
                break;
            }
        }

        if reasons.is_empty() {
            "相关匹配".to_string()
        } else {
            reasons.join(", ")
        }
    }

    /// 应用搜索过滤器
    fn apply_search_filters(
        &self,
        mut interfaces: Vec<InterfaceWithScore>,
        filters: &InterfaceSearchFilters,
        project_id: &Option<String>,
    ) -> Vec<InterfaceWithScore> {
        interfaces.retain(|item| {
            // 项目ID过滤
            if let Some(_pid) = project_id {
                // 这里需要从数据库查询项目ID，暂时跳过
            }

            // HTTP方法过滤
            if let Some(methods) = &filters.methods {
                if !methods.contains(&item.interface.method) {
                    return false;
                }
            }

            // 标签过滤
            if let Some(filter_tags) = &filters.tags {
                if !filter_tags
                    .iter()
                    .any(|tag| item.interface.tags.contains(tag))
                {
                    return false;
                }
            }

            // 域过滤
            if let Some(domain) = &filters.domain {
                if item.interface.domain.as_ref() != Some(domain) {
                    return false;
                }
            }

            // 是否包含已弃用的接口
            if !filters.include_deprecated.unwrap_or(true) && item.interface.deprecated {
                return false;
            }

            // 路径前缀过滤
            if let Some(prefix) = &filters.path_prefix {
                if !item.interface.path.starts_with(prefix) {
                    return false;
                }
            }

            true
        });

        interfaces
    }

    /// 获取项目的所有接口
    pub async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<ApiInterface>> {
        let query_str = format!(
            "SELECT * FROM interface WHERE project_id = '{}'",
            project_id
        );
        let mut response = self.db.query(&query_str).await?;
        let records: Vec<InterfaceRecord> = response.take(0)?;

        Ok(records.into_iter().map(|r| r.interface).collect())
    }

    /// 删除项目数据
    pub async fn delete_project_data(&self, project_id: &str) -> Result<String> {
        let query_str = format!("DELETE FROM interface WHERE project_id = '{}'", project_id);
        self.db.query(&query_str).await?;

        Ok(format!(
            "Deleted all interfaces for project: {}",
            project_id
        ))
    }
}
