use crate::config::EmbeddingConfig;
use crate::models::interface_retrieval::*;
use crate::services::interface_retrieval_service::InterfaceRetrievalService;
use crate::services::EmbeddingService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// 接口关系处理器的应用状态
#[derive(Clone)]
pub struct InterfaceRetrievalState {
    pub retrieval: Arc<InterfaceRetrievalService>,
}

impl InterfaceRetrievalState {
    pub async fn new(
        embedding_config: EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let service = Arc::new(
            InterfaceRetrievalService::new(
                &embedding_config,
                embedding_service,
            )
            .await?,
        );
        Ok(Self { retrieval: service })
    }
}

/// 创建接口关系路由
pub fn create_interface_relation_routes() -> Router<InterfaceRetrievalState> {
    Router::new()
        .route(
            "/api/interface-retrieval/swagger/parse",
            post(parse_swagger_json),
        )
        .route("/api/interface-retrieval/search", post(search_interfaces))
        .route(
            "/api/interface-retrieval/projects/{project_id}",
            delete(delete_project_data),
        )
}

/// 删除项目数据
///
/// 删除指定项目的所有接口和依赖关系数据
pub async fn delete_project_data(
    State(state): State<InterfaceRetrievalState>,
    Path(project_id): Path<String>,
) -> Result<Json<bool>, (StatusCode, Json<InterfaceRelationError>)> {
    tracing::info!("Deleting data for project: {}", project_id);

    if project_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(InterfaceRelationError {
                code: "INVALID_PROJECT_ID".to_string(),
                message: "项目ID不能为空".to_string(),
                details: None,
            }),
        ));
    }

    match state.retrieval.delete_project_data(&project_id).await {
        Ok(result) => {
            tracing::info!("Successfully deleted project data(vector): {}", result);
            Ok(Json(true))
        }
        Err(e) => {
            tracing::error!("Failed to delete project data: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InterfaceRelationError {
                    code: "DELETE_ERROR".to_string(),
                    message: "删除项目数据失败".to_string(),
                    details: Some({
                        let mut details = HashMap::new();
                        details.insert("error".to_string(), e.to_string());
                        details.insert("project_id".to_string(), project_id);
                        details
                    }),
                }),
            ))
        }
    }
}

/// 解析Swagger JSON数据
///
/// 接收Swagger JSON格式数据，解析其中的HTTP接口信息并存储到数据库
pub async fn parse_swagger_json(
    State(state): State<InterfaceRetrievalState>,
    Json(request): Json<SwaggerParseRequest>,
) -> Result<Json<bool>, (StatusCode, Json<InterfaceRelationError>)> {
    tracing::info!("Parsing Swagger JSON for project: {}", request.project_id);
    let _start_time = Instant::now();

    // 验证请求数据
    if request.project_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(InterfaceRelationError {
                code: "INVALID_PROJECT_ID".to_string(),
                message: "项目ID不能为空".to_string(),
                details: None,
            }),
        ));
    }
    match state.retrieval.parse_and_store_swagger(request).await {
        Ok(_) => Ok(Json(true)),
        Err(e) => {
            tracing::error!("Failed to parse Swagger JSON: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InterfaceRelationError {
                    code: "SWAGGER_PARSE_ERROR".to_string(),
                    message: format!("解析Swagger JSON失败: {}", e),
                    details: None,
                }),
            ))
        }
    }
}

/// 搜索接口信息
///
/// 通过关键词向量或完全匹配方式检索相关接口信息
pub async fn search_interfaces(
    State(state): State<InterfaceRetrievalState>,
    Json(request): Json<InterfaceSearchRequest>,
) -> Result<Json<InterfaceSearchResponse>, (StatusCode, Json<InterfaceRelationError>)> {
    tracing::info!("Searching interfaces with query: {}", request.query);

    // 验证请求数据
    if request.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(InterfaceRelationError {
                code: "EMPTY_QUERY".to_string(),
                message: "搜索查询不能为空".to_string(),
                details: None,
            }),
        ));
    }

    let start_time = Instant::now();
    let search_type = request.search_type.clone();
    match state.retrieval.search_interfaces(request).await {
        Ok(chunks) => {
            let mut interfaces_with_score = Vec::new();
            for chunk in &chunks {
                let project_id = chunk
                    .meta
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // 直接使用chunk中的api_content字段
                if let Some(api_interface) = &chunk.api_content {
                    // 创建InterfaceWithScore
                    let interface_with_score = InterfaceWithScore {
                        project_id: Some(project_id.to_string()),
                        interface: api_interface.clone(),
                        score: chunk.score,
                        match_reason: format!(
                            "向量搜索匹配: {} {}",
                            api_interface.method, api_interface.path
                        ),
                    };

                    interfaces_with_score.push(interface_with_score);
                } else {
                    tracing::debug!("Chunk {} has no api_content, skipping", chunk.id);
                }
            }

            let query_time_ms = start_time.elapsed().as_millis() as u64;
            let total_count = interfaces_with_score.len() as u32;

            // 构建响应
            let response = InterfaceSearchResponse {
                interfaces: interfaces_with_score,
                query_time_ms,
                total_count,
                search_mode: format!("{:?}", search_type),
            };

            tracing::info!(
                "Interface search completed: {} results found in {}ms",
                response.total_count,
                response.query_time_ms
            );

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to search interfaces: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InterfaceRelationError {
                    code: "SEARCH_ERROR".to_string(),
                    message: format!("搜索接口失败: {}", e),
                    details: None,
                }),
            ))
        }
    }
}
