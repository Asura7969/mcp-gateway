use crate::models::endpoint::Endpoint;
use crate::services::{
    interface_relation_service::{InterfaceRelationService, ParseSwaggerRequest},
    EndpointService,
};
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};

/// 启动时自动加载服务
pub struct StartupLoaderService {
    endpoint_service: Arc<EndpointService>,
    interface_relation_service: Arc<InterfaceRelationService>,
}

impl StartupLoaderService {
    /// 创建新的启动加载服务实例
    pub fn new(
        endpoint_service: Arc<EndpointService>,
        interface_relation_service: Arc<InterfaceRelationService>,
    ) -> Self {
        Self {
            endpoint_service,
            interface_relation_service,
        }
    }

    /// 自动加载所有endpoints的swagger信息到SurrealDB
    pub async fn load_all_swagger_data(&self) -> Result<()> {
        info!("开始自动加载endpoints表中的swagger信息到SurrealDB...");

        // 获取所有endpoints
        let endpoints = match self.endpoint_service.get_all_endpoints().await {
            Ok(endpoints) => endpoints,
            Err(e) => {
                error!("获取endpoints失败: {}", e);
                return Err(anyhow::anyhow!("获取endpoints失败: {}", e));
            }
        };

        if endpoints.is_empty() {
            info!("没有找到任何endpoints，跳过swagger数据加载");
            return Ok(());
        }

        info!("找到 {} 个endpoints，开始处理swagger数据", endpoints.len());

        let mut success_count = 0;
        let mut error_count = 0;

        // 逐个处理每个endpoint的swagger信息
        for endpoint in endpoints {
            match self.process_endpoint_swagger(&endpoint).await {
                Ok(()) => {
                    success_count += 1;
                    info!("成功处理endpoint: {} (ID: {})", endpoint.name, endpoint.id);
                }
                Err(e) => {
                    error_count += 1;
                    error!(
                        "处理endpoint失败: {} (ID: {}), 错误: {}",
                        endpoint.name, endpoint.id, e
                    );
                }
            }
        }

        info!(
            "swagger数据加载完成: 成功 {} 个, 失败 {} 个",
            success_count, error_count
        );

        if error_count > 0 {
            warn!("有 {} 个endpoints的swagger数据加载失败", error_count);
        }

        Ok(())
    }

    /// 处理单个endpoint的swagger信息
    async fn process_endpoint_swagger(&self, endpoint: &Endpoint) -> Result<()> {
        // 检查swagger_content是否为空
        if endpoint.swagger_content.is_empty() {
            warn!("Endpoint {} 的swagger_content为空，跳过处理", endpoint.name);
            return Ok(());
        }

        // 解析swagger JSON
        let swagger_json: serde_json::Value = match serde_json::from_str(&endpoint.swagger_content)
        {
            Ok(json) => json,
            Err(e) => {
                error!("解析endpoint {} 的swagger JSON失败: {}", endpoint.name, e);
                return Err(anyhow::anyhow!("Swagger JSON解析失败: {}", e));
            }
        };

        // 使用endpoint的name作为project_id
        let project_id = endpoint.name.clone();
        let request = ParseSwaggerRequest {
            swagger_json,
            project_id: project_id.clone(),
        };

        info!(
            "Processing endpoint: {}, project_id: {}",
            endpoint.name, project_id
        );

        // 调用interface_relation_service的parse_and_store_swagger方法
        match self
            .interface_relation_service
            .parse_and_store_swagger(request)
            .await
        {
            Ok(_) => {
                info!("Endpoint {} swagger解析成功", endpoint.name);
                Ok(())
            }
            Err(e) => {
                error!("调用parse_swagger_json失败: {}", e);
                Err(anyhow::anyhow!("调用parse_swagger_json失败: {}", e))
            }
        }
    }
}
