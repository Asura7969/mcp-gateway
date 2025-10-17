use crate::models::interface_retrieval::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// 搜索服务的通用trait，支持向量检索、关键词检索和混合检索
#[async_trait]
pub trait Search: Send + Sync {
    /// 解析并存储Swagger文档
    async fn parse_and_store_swagger(&self, request: SwaggerParseRequest) -> Result<()>;

    /// 向量搜索 - 基于语义相似度
    async fn vector_search(
        &self,
        query: &str,
        max_results: u32,
        similarity_threshold: f32,
        filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>>;

    /// 关键词搜索 - 基于文本匹配
    async fn keyword_search(
        &self,
        query: &str,
        max_results: u32,
        filters: Option<&Filter>,
    ) -> Result<Vec<Chunk>>;

    /// 混合搜索 - 结合向量搜索和关键词搜索
    async fn hybrid_search(
        &self,
        request: InterfaceSearchRequest
    ) -> Result<Vec<Chunk>>;

    /// 获取项目的所有接口
    async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<Chunk>>;

    /// 删除项目数据
    async fn delete_project_data(&self, project_id: &str) -> Result<u64>;

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Uuid,
    pub text: String,
    pub meta: Value,
    pub score: f64,
    pub embedding: Vec<f32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Filter {
    pub project_id: Option<String>,
    // 路径前置过滤
    pub prefix_path: Option<String>,
    pub methods: Option<Vec<String>>,
}

pub fn merge_content(interface: &ApiInterface) -> String {
    format!("{} | {} | {}",
            &interface.summary.clone().unwrap_or("".to_string()),
            &interface.description.clone().unwrap_or("".to_string()),
            &interface.service_description.clone().unwrap_or("".to_string()))
}