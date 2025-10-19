use crate::config::EmbeddingConfig;
use crate::models::interface_retrieval::*;
use crate::models::DbPool;
use crate::services::interface_retrieval_service::InterfaceRetrievalService;
use crate::services::EmbeddingService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// 项目信息结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

/// 接口关系处理器的应用状态
#[derive(Clone)]
pub struct InterfaceRetrievalState {
    pub retrieval: Arc<InterfaceRetrievalService>,
    pub db_pool: DbPool,
}

impl InterfaceRetrievalState {
    pub async fn new(
        embedding_config: EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
        db_pool: DbPool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let service = Arc::new(
            InterfaceRetrievalService::new(
                &embedding_config,
                embedding_service,
            )
            .await?,
        );
        Ok(Self { 
            retrieval: service,
            db_pool,
        })
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
        .route("/api/interface-retrieval/projects", get(get_projects))
        .route(
            "/api/interface-retrieval/projects/{project_id}",
            delete(delete_project_data),
        )
}

/// 获取项目列表
pub async fn get_projects(
    State(state): State<InterfaceRetrievalState>,
) -> Result<Json<Vec<ProjectInfo>>, StatusCode> {
    let query = "SELECT DISTINCT name, name as id, 'active' as status FROM endpoints ORDER BY name";
    
    match sqlx::query_as::<_, (String, String, String)>(query)
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(rows) => {
            let projects: Vec<ProjectInfo> = rows
                .into_iter()
                .map(|(name, id, status)| ProjectInfo {
                    id,
                    name: name.clone(),
                    description: Some(format!("Project: {}", name)),
                    status,
                })
                .collect();
            Ok(Json(projects))
        }
        Err(e) => {
            tracing::error!("Failed to fetch projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 删除项目数据
pub async fn delete_project_data(
    State(state): State<InterfaceRetrievalState>,
    Path(project_id): Path<String>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    match state.retrieval.delete_project_data(&project_id).await {
        Ok(_) => {
            let mut response = HashMap::new();
            response.insert("message".to_string(), "Project data deleted successfully".to_string());
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to delete project data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
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
