use crate::config::EmbeddingConfig;
use crate::models::interface_retrieval::*;
use crate::models::swagger::SwaggerSpec;
use crate::services::{merge_content, Chunk, EmbeddingService, Filter, Search};
use crate::utils::generate_api_details;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Pool, Postgres, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

impl From<&PgRow> for Chunk {
    fn from(row: &PgRow) -> Self {
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");
        let api_content: String = row.get("api_content");
        let api_content = Some(serde_json::from_str::<ApiInterface>(api_content.as_str()).unwrap());
        Self {
            id: row.get("id"),
            text: row.get("text"),
            meta: row.get("meta"),
            score: row.get("score"),
            embedding: Vec::with_capacity(0),
            api_content,
            created_at: Some(created_at),
            updated_at: Some(updated_at),
        }
    }
}

#[derive(Debug)]
enum ParamValue {
    I64(i64),
    Text(String),
    // 添加更多类型...
}

/// PgVector-RS 向量检索服务
pub struct PgvectorRsSearch {
    pool: Pool<Postgres>,
    embedding_service: Arc<EmbeddingService>,
}

impl PgvectorRsSearch {
    /// 创建新的PgvectorRsService实例
    pub async fn new(
        config: &EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self> {
        let pgvector_config = config
            .pgvectorrs
            .as_ref()
            .ok_or_else(|| anyhow!("PgVector-RS configuration not found"))?;

        let db_connection_str = format!(
            "postgres://{}:{}@{}:{}/{}",
            pgvector_config.user,
            pgvector_config.password,
            pgvector_config.host,
            pgvector_config.port,
            pgvector_config.database
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        let service = Self {
            pool,
            embedding_service,
        };

        // 初始化数据库schema
        service.init_schema().await?;

        Ok(service)
    }

    /// 初始化数据库schema
    async fn init_schema(&self) -> Result<()> {
        // 创建pgvecto-rs扩展
        sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS vectors"#)
            .execute(&self.pool)
            .await?;

        // meta: project_id, method, path,
        // embedding: summary, description, service_description
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS interfaces_v2 (
                id UUID PRIMARY KEY,
                text TEXT NOT NULL,
                api_content TEXT NOT NULL,
                text_tsvector TSVECTOR DEFAULT NULL,
                meta JSONB NOT NULL,
                embedding vector(1024) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            ) using heap;
        "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_embedding
            ON interfaces_v2 USING vectors(embedding vector_l2_ops)
            WITH (options = $$
                    optimizing.optimizing_threads = 30
                    segment.max_growing_segment_size = 2000
                    segment.max_sealed_segment_size = 30000000
                    [indexing.hnsw]
                    m=30
                    ef_construction=500
                    $$);
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_fts ON interfaces_v2 USING GIN (text_tsvector)
        "#,
        )
        .execute(&self.pool)
        .await?;
        // todo: 添加meta字段索引

        info!("PgVector-RS schema initialized successfully");
        Ok(())
    }

    /// 存储接口到数据库
    async fn store_interfaces(&self, interfaces: &[ApiInterface], project_id: &str) -> Result<u64> {
        let mut stored_count = 0;

        for interface in interfaces {
            // 插入或更新接口
            let meta_value = json!({
                "project_id": project_id,
                "method": interface.method,
                "path": interface.path
            });

            let text = merge_content(interface);
            let embedding = self.embedding_service.embed_text(&text).await?;
            let api_content = serde_json::to_string::<ApiInterface>(interface).unwrap();

            let result = sqlx::query(
                "
                INSERT INTO interfaces_v2 (
                    id, text, text_tsvector, meta, embedding, created_at, updated_at, api_content
                ) VALUES ($1, $2, to_tsvector('chinese_zh', $3), $4, $5, NOW(), NOW(), $6)
                ",
            )
            .bind(Uuid::new_v4())
            .bind(text.clone())
            .bind(text)
            .bind(meta_value)
            .bind(embedding)
            .bind(api_content)
            .execute(&self.pool)
            .await?;

            stored_count += result.rows_affected()
        }

        Ok(stored_count)
    }
}

#[async_trait]
impl Search for PgvectorRsSearch {
    async fn parse_and_store_swagger(&self, request: SwaggerParseRequest) -> Result<()> {
        info!("Parsing Swagger for project: {}", request.project_id);

        // 解析Swagger JSON
        let swagger_spec: SwaggerSpec = serde_json::from_value(request.swagger_json)?;
        let api_details = generate_api_details(&swagger_spec)?;

        info!("Found {} interfaces in Swagger", api_details.len());

        // 将ApiDetail转换为ApiInterface
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

    async fn vector_search(
        &self,
        query: &str,
        max_results: u32,
        _similarity_threshold: f32,
        _filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>> {
        // 获取查询向量
        let query_embedding = self.embedding_service.embed_text(query).await?;

        // 构建SQL查询
        // let query_vector_str = format!("[{}]", query_embedding.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","));
        let sql = r#"
            SELECT *, embedding <=> $1 AS score
            FROM interfaces_v2
            ORDER BY score
            LIMIT $2
        "#
        .to_string();

        // let mut param_count = 1;
        // let mut boxed_params: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> = vec![
        //     Box::new(similarity_threshold as f64),
        // ];
        //
        // // 添加项目ID过滤
        // let project_id_owned = project_id.map(|s| s.to_string());
        // if let Some(ref pid) = project_id_owned {
        //     param_count += 1;
        //     sql.push_str(&format!(" AND project_id = ${}", param_count));
        //     boxed_params.push(Box::new(pid.clone()));
        // }
        //
        // // 添加过滤条件
        // let filter_conditions = self.build_filter_conditions(filters, &mut param_count, &mut boxed_params);
        // sql.push_str(&filter_conditions);
        //
        // // 添加排序和限制
        // sql.push_str(" ORDER BY similarity DESC");
        // param_count += 1;
        // sql.push_str(&format!(" LIMIT ${}", param_count));
        // let limit_value = max_results as i64;
        // boxed_params.push(Box::new(limit_value));
        //
        // // 转换为引用参数
        // let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = boxed_params
        //     .iter()
        //     .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        //     .collect();

        // 执行查询
        let rows = sqlx::query(&sql)
            .bind(query_embedding)
            .bind(max_results as i64)
            .fetch_all(&self.pool)
            .await?;

        let results: Vec<Chunk> = rows.iter().map(Chunk::from).collect();

        Ok(results)
    }

    async fn keyword_search(
        &self,
        query: &str,
        max_results: u32,
        filter: Option<&Filter>,
    ) -> Result<Vec<Chunk>> {
        let mut params = vec![ParamValue::Text(query.to_string())];
        let mut sql = r#"
            SELECT
                id, text, meta, created_at, updated_at, api_content,
                ts_rank(text_tsvector, websearch_to_tsquery('chinese_zh', $1)) AS score
            FROM interfaces_v2
        "#
        .to_string();
        let mut param_count = 2;

        let mut condition_sql = vec![];
        if let Some(condition) = filter {
            if let Some(project_id) = &condition.project_id {
                params.push(ParamValue::Text(project_id.to_string()));
                let c = format!(" meta->>'project_id' = ${} ", param_count);
                condition_sql.push(c);
                param_count += 1;
            }

            if let Some(prefix_path) = &condition.prefix_path {
                let mut path = prefix_path.to_string();
                path.push_str("%");
                params.push(ParamValue::Text(path));
                let c = format!(" meta->>'path' LIKE ${} ", param_count);
                condition_sql.push(c);
                param_count += 1;
            }

            if let Some(methods) = &condition.methods {
                params.push(ParamValue::Text(methods.join(", ")));
                let c = format!(" meta->>'method' in (${}) ", param_count);
                condition_sql.push(c);
                param_count += 1;
            }
        }

        if !condition_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(condition_sql.join(" AND ").as_str());
        }

        sql.push_str(&format!(" ORDER BY score DESC LIMIT ${}", param_count));
        params.push(ParamValue::I64(max_results as i64));

        let mut query = sqlx::query(&sql);
        for param in params {
            match param {
                ParamValue::I64(val) => query = query.bind(val),
                ParamValue::Text(val) => query = query.bind(val),
            }
        }

        // 执行查询
        let rows = query.fetch_all(&self.pool).await?;

        let results = rows.iter().map(Chunk::from).collect::<Vec<Chunk>>();

        Ok(results)
    }

    async fn hybrid_search(&self, request: InterfaceSearchRequest) -> Result<Vec<Chunk>> {
        // 执行向量搜索，传递过滤器
        let vector_results = self
            .vector_search(
                request.query.as_str(),
                request.max_results * 2,
                request.similarity_threshold.unwrap_or(0.5),
                request.filters.as_ref(),
            )
            .await?;

        let (vector_weight, _) = match &request.vector_weight {
            None => (0.0f32, 1f32),
            Some(vector_weight) => (*vector_weight, 1.0 - vector_weight),
        };

        // 执行关键词搜索，传递过滤器
        let keyword_results = self
            .keyword_search(
                request.query.as_str(),
                request.max_results * 2,
                request.filters.as_ref(),
            )
            .await?;

        // 合并结果并计算混合分数
        let mut combined_results: HashMap<String, Chunk> = HashMap::new();

        // 添加向量搜索结果
        for chunk in vector_results {
            let key = chunk.id.to_string();
            let hybrid_score = chunk.score * vector_weight as f64;
            let mut hybrid_result = chunk;
            hybrid_result.score = hybrid_score;
            combined_results.insert(key, hybrid_result);
        }

        // 添加关键词搜索结果
        for chunk in keyword_results {
            let key = chunk.id.to_string();
            let keyword_score = chunk.score * (1.0 - vector_weight as f64);

            if let Some(existing) = combined_results.get_mut(key.as_str()) {
                // 合并分数
                existing.score += keyword_score;
            } else {
                let mut hybrid_result = chunk;
                hybrid_result.score = keyword_score;
                combined_results.insert(key, hybrid_result);
            }
        }

        // 转换为向量并排序
        let mut results: Vec<Chunk> = combined_results.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制结果数量
        results.truncate(request.max_results as usize);

        Ok(results)
    }

    async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<Chunk>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM interfaces_v2 WHERE meta->>'project_id' = $1 ORDER BY path, method
        "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let result = rows.iter().map(Chunk::from).collect::<Vec<Chunk>>();
        Ok(result)
    }

    async fn delete_project_data(&self, project_id: &str) -> Result<u64> {
        let pqr = sqlx::query(r#"DELETE FROM interfaces_v2 WHERE meta->>'project_id' = $1"#)
            .bind(project_id)
            .execute(&self.pool)
            .await?;
        Ok(pqr.rows_affected())
    }
}
