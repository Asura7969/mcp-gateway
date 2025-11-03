use crate::models::table_rag::FileMeta;
use crate::services::FileService;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileState {
    pub service: Arc<FileService>,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub files: Vec<FileMeta>,
}

pub async fn upload_files_handler(
    State(state): State<FileState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    let mut results: Vec<FileMeta> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        let meta = state
            .service
            .upload_and_save(&name, data.to_vec())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        results.push(meta);
    }

    Ok(Json(UploadResponse { files: results }))
}
