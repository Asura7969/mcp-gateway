use crate::config::{EmbeddingConfig, VectorType};
use crate::models::interface_retrieval::*;
use crate::services::{ElasticSearch, EmbeddingService, PgvectorRsSearch, Search};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

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
    pub async fn search_interfaces(
        &self,
        request: InterfaceSearchRequest,
    ) -> Result<InterfaceSearchResponse> {
        let _start_time = Instant::now();
        let chunks = self.search.hybrid_search(request).await?;
        // todo: 依据chunks 查询数据库接口详情
        let _total_count = chunks.len() as u32;

        // Ok(InterfaceSearchResponse {
        //     interfaces,
        //     query_time_ms: start_time.elapsed().as_millis() as u64,
        //     total_count,
        //     search_mode,
        // });

        todo!()
    }

    /// 获取项目的所有接口
    pub async fn get_project_interfaces(&self, project_id: &str) -> Result<Vec<ApiInterface>> {
        let _chunks = self.search.get_project_interfaces(project_id).await?;
        //
        // Ok(records.into_iter().map(|r| r.interface).collect())

        todo!()
    }

    /// 删除项目数据
    pub async fn delete_project_data(&self, project_id: &str) -> Result<String> {
        let count = self.search.delete_project_data(project_id).await?;
        Ok(count.to_string())
    }
}
