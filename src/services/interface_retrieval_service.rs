use crate::config::{EmbeddingConfig, VectorType};
use crate::models::interface_retrieval::*;
use crate::services::{Chunk, ElasticSearch, EmbeddingService, Meta, PgvectorRsSearch, Search};
use anyhow::Result;
use std::sync::Arc;

/// 接口关系服务 - 重新设计用于swagger解析和向量搜索
pub struct InterfaceRetrievalService {
    search: Box<dyn Search>,
}

impl InterfaceRetrievalService {
    /// 创建新的服务实例
    pub async fn new(
        config: &EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self> {
        let search: Box<dyn Search> = match config.vector_type {
            VectorType::Elasticsearch => {
                Box::new(ElasticSearch::new(config, embedding_service.clone()).await?)
            }
            VectorType::PgVectorRs => {
                Box::new(PgvectorRsSearch::new(config, embedding_service.clone()).await?)
            }
        };
        let service = Self { search };
        Ok(service)
    }

    /// 解析Swagger JSON并存储接口信息
    pub async fn parse_and_store_swagger(&self, request: SwaggerParseRequest) -> Result<()> {
        self.search.parse_and_store_swagger(request).await
    }

    /// 搜索接口 - 支持关键词和向量搜索
    pub async fn search_interfaces(&self, request: InterfaceSearchRequest) -> Result<Vec<Chunk>> {
        Ok(self.search.hybrid_search(request).await?)
    }

    /// 获取项目的所有接口
    pub async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<ApiInterface>> {
        let chunks = self.search.get_project_interfaces(project_id).await?;

        // 从chunks中提取ApiInterface
        let interfaces = chunks
            .into_iter()
            .filter_map(|chunk| chunk.api_content)
            .collect();

        Ok(interfaces)
    }

    /// 删除项目数据
    pub async fn delete_project_data(&self, project_id: &str) -> Result<String> {
        let count = self.search.delete_project_data(project_id).await?;
        Ok(count.to_string())
    }

    pub async fn update(&self, interface: &ApiInterface, project_id: String) -> Result<()> {
        let meta = Meta {
            project_id: project_id.clone(),
            path: interface.path.clone(),
            method: interface.method.clone(),
        };
        self.search.delete_by_meta(meta).await?;
        self.search
            .store_interface(interface.clone(), project_id)
            .await?;
        Ok(())
    }
}
