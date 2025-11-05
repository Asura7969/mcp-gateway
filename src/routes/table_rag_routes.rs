use crate::handlers::{
    create_dataset_handler, get_dataset_handler, ingest_dataset_file_handler,
    list_datasets_handler, list_remote_tables_handler, list_tasks_handler, preview_schema_handler,
    search_handler, search_paged_handler, test_remote_connection_handler, update_dataset_handler,
    TableRagState,
};
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_table_rag_routes() -> Router<TableRagState> {
    Router::new()
        .route(
            "/api/table-rag/datasets",
            post(create_dataset_handler).get(list_datasets_handler),
        )
        .route(
            "/api/table-rag/datasets/{id}",
            get(get_dataset_handler).put(update_dataset_handler),
        )
        .route("/api/table-rag/ingest", post(ingest_dataset_file_handler))
        .route(
            "/api/table-rag/preview-schema",
            post(preview_schema_handler),
        )
        .route("/api/table-rag/search", post(search_handler))
        .route("/api/table-rag/search-paged", post(search_paged_handler))
        .route("/api/table-rag/tasks", get(list_tasks_handler))
        .route(
            "/api/table-rag/remote/test-connection",
            post(test_remote_connection_handler),
        )
        .route(
            "/api/table-rag/remote/list-tables",
            post(list_remote_tables_handler),
        )
}
