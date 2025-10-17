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
use crate::config::EmbeddingConfig;

/// 接口关系处理器的应用状态
#[derive(Clone)]
pub struct InterfaceRetrievalState {
    pub service: Arc<InterfaceRetrievalService>,
}

impl InterfaceRetrievalState {
    pub async fn new(
        embedding_config: EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let service = Arc::new(InterfaceRetrievalService::new(&embedding_config, embedding_service).await?);
        Ok(Self { service })
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
) -> Result<Json<String>, (StatusCode, Json<InterfaceRelationError>)> {
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

    match state.service.delete_project_data(&project_id).await {
        Ok(result) => {
            tracing::info!("Successfully deleted project data: {}", result);
            Ok(Json(result))
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
    match state.service.parse_and_store_swagger(request).await {
        Ok(_) => {
            Ok(Json(true))
        }
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

    match state.service.search_interfaces(request).await {
        Ok(response) => {
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
