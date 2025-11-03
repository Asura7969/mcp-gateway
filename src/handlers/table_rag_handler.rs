use axum::extract::{Path, Query};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::table_rag::{
    ColumnSchema, CreateDatasetRequest, DatasetDetailResponse, DatasetResponse,
    UpdateDatasetRequest,
};
use crate::services::TableRagService;

#[derive(Clone)]
pub struct TableRagState {
    pub service: Arc<TableRagService>,
}

#[derive(Debug, Deserialize)]
pub struct IngestPathParams {
    pub dataset_id: String,
    pub file_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TableSearchRequest {
    pub dataset_id: String,
    pub query: String,
    pub max_results: Option<u32>,
    pub similarity_threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct IngestResult {
    pub ingested_rows: u32,
    pub task_id: Option<String>,
}

pub async fn create_dataset_handler(
    State(state): State<TableRagState>,
    Json(req): Json<CreateDatasetRequest>,
) -> Result<Json<DatasetResponse>, (StatusCode, String)> {
    state
        .service
        .create_dataset(req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct ListDatasetsQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn list_datasets_handler(
    State(state): State<TableRagState>,
    Query(query): Query<ListDatasetsQuery>,
) -> Result<Json<Vec<DatasetResponse>>, (StatusCode, String)> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    state
        .service
        .list_datasets_paged(page, page_size)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_dataset_handler(
    State(state): State<TableRagState>,
    Path(id): Path<String>,
) -> Result<Json<DatasetDetailResponse>, (StatusCode, String)> {
    let dataset_id = Uuid::parse_str(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid dataset_id: {}", e),
        )
    })?;
    state
        .service
        .get_dataset_by_id(dataset_id)
        .await
        .map(|d| Json(d.into()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn update_dataset_handler(
    State(state): State<TableRagState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDatasetRequest>,
) -> Result<Json<DatasetResponse>, (StatusCode, String)> {
    let dataset_id = Uuid::parse_str(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid dataset_id: {}", e),
        )
    })?;
    state
        .service
        .update_dataset(dataset_id, req)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn ingest_dataset_file_handler(
    State(state): State<TableRagState>,
    Json(params): Json<IngestPathParams>,
) -> Result<Json<IngestResult>, (StatusCode, String)> {
    let dataset_id = Uuid::parse_str(&params.dataset_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid dataset_id: {}", e),
        )
    })?;
    let file_id = Uuid::parse_str(&params.file_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid file_id: {}", e)))?;
    // 两段式：先创建任务，再后台执行
    let task_id = state
        .service
        .create_ingest_task(dataset_id, file_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let service = state.service.clone();
    tokio::spawn(async move {
        if let Err(err) = service.run_ingest_task(task_id).await {
            tracing::error!("table_rag ingest task failed: {}", err);
        }
    });
    Ok(Json(IngestResult {
        ingested_rows: 0,
        task_id: Some(task_id.to_string()),
    }))
}

pub async fn search_handler(
    State(state): State<TableRagState>,
    Json(req): Json<TableSearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let dataset_id = Uuid::parse_str(&req.dataset_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid dataset_id: {}", e),
        )
    })?;
    // If max_results is not provided, let service decide based on dataset defaults
    let max = req.max_results.unwrap_or(0);
    state
        .service
        .search(dataset_id, &req.query, max, req.similarity_threshold)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct PreviewSchemaRequest {
    pub file_ids: Vec<String>,
}

pub async fn preview_schema_handler(
    State(state): State<TableRagState>,
    Json(req): Json<PreviewSchemaRequest>,
) -> Result<Json<Vec<ColumnSchema>>, (StatusCode, String)> {
    if req.file_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "file_ids cannot be empty".to_string(),
        ));
    }
    let mut ids = Vec::new();
    for id_str in req.file_ids {
        match Uuid::parse_str(&id_str) {
            Ok(id) => ids.push(id),
            Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Invalid file_id: {}", e))),
        }
    }
    state
        .service
        .preview_schema_from_files(ids)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub dataset_id: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn list_tasks_handler(
    State(state): State<TableRagState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<Vec<crate::models::table_rag::IngestTask>>, (StatusCode, String)> {
    let dataset_id = Uuid::parse_str(&query.dataset_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid dataset_id: {}", e),
        )
    })?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    state
        .service
        .list_tasks_by_dataset(dataset_id, page, page_size)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct RemoteDbRequest {
    pub driver: Option<String>, // 支持: mysql
    pub url: String,            // 例如: mysql://user:pass@host:3306/db
}

pub async fn test_remote_connection_handler(
    State(state): State<TableRagState>,
    Json(req): Json<RemoteDbRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let driver = req.driver.unwrap_or_else(|| "mysql".to_string());
    match driver.as_str() {
        "mysql" => {
            state
                .service
                .test_remote_connection_mysql(&req.url)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            Ok(Json(serde_json::json!({"ok": true})))
        }
        _ => Err((StatusCode::BAD_REQUEST, "unsupported driver".to_string())),
    }
}

pub async fn list_remote_tables_handler(
    State(state): State<TableRagState>,
    Json(req): Json<RemoteDbRequest>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let driver = req.driver.unwrap_or_else(|| "mysql".to_string());
    match driver.as_str() {
        "mysql" => state
            .service
            .list_remote_tables_mysql(&req.url)
            .await
            .map(Json)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string())),
        _ => Err((StatusCode::BAD_REQUEST, "unsupported driver".to_string())),
    }
}
