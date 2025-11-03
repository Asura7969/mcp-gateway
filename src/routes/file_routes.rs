use crate::handlers::{upload_files_handler, FileState};
use axum::{routing::post, Router};

pub fn create_file_routes() -> Router<FileState> {
    Router::new().route("/api/files/upload", post(upload_files_handler))
}
