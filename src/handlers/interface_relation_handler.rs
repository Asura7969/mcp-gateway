use crate::models::interface_relation::*;
use crate::services::interface_relation_service::{InterfaceRelationService, ParseSwaggerRequest};
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
pub struct InterfaceRelationState {
    pub service: Arc<InterfaceRelationService>,
}

impl InterfaceRelationState {
    pub async fn new(
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let service = Arc::new(InterfaceRelationService::new(embedding_service).await?);
        Ok(Self { service })
    }
}

/// 创建接口关系路由
pub fn create_interface_relation_routes() -> Router<InterfaceRelationState> {
    Router::new()
        .route(
            "/api/interface-relations/swagger/parse",
            post(parse_swagger_json),
        )
        .route("/api/interface-relations/search", post(search_interfaces))
        .route(
            "/api/interface-relations/projects/{project_id}",
            delete(delete_project_data),
        )
}

/// 删除项目数据
///
/// 删除指定项目的所有接口和依赖关系数据
pub async fn delete_project_data(
    State(state): State<InterfaceRelationState>,
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
    State(state): State<InterfaceRelationState>,
    Json(request): Json<SwaggerParseRequest>,
) -> Result<Json<SwaggerParseResponse>, (StatusCode, Json<InterfaceRelationError>)> {
    tracing::info!("Parsing Swagger JSON for project: {}", request.project_id);
    let start_time = Instant::now();

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

    // 转换请求格式
    let service_request = ParseSwaggerRequest {
        project_id: request.project_id.clone(),
        swagger_json: request.swagger_json,
    };

    match state.service.parse_and_store_swagger(service_request).await {
        Ok(_) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            // 获取项目接口数量作为存储数量的估算
            let stored_count = match state
                .service
                .get_project_interfaces(&request.project_id)
                .await
            {
                Ok(interfaces) => interfaces.len() as u32,
                Err(_) => 0,
            };

            let response = SwaggerParseResponse {
                parsed_interfaces_count: stored_count,
                stored_interfaces_count: stored_count,
                dependencies_count: 0, // 暂时不支持依赖关系
                processing_time_ms: processing_time,
                errors: vec![],
                warnings: vec![],
            };

            tracing::info!(
                "Swagger parsing completed: {} interfaces stored in {}ms",
                stored_count,
                processing_time
            );
            Ok(Json(response))
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
    State(state): State<InterfaceRelationState>,
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
