use crate::models::{
    CreateEndpointRequest, EndpointDetailResponse, EndpointMetrics, EndpointQueryParams,
    EndpointResponse, PaginatedEndpointsResponse, PaginationInfo, UpdateEndpointRequest,
};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;

// #[utoipa::path(
//     post,
//     path = "/api/endpoint",
//     request_body = CreateEndpointRequest,
//     responses(
//         (status = 201, description = "Endpoint created successfully", body = EndpointResponse),
//         (status = 400, description = "Bad request"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn create_endpoint(
    State(app_state): State<AppState>,
    Json(request): Json<CreateEndpointRequest>,
) -> Result<(StatusCode, Json<EndpointResponse>), (StatusCode, String)> {
    match app_state.endpoint_service.create_endpoint(request).await {
        Ok(endpoint) => Ok((StatusCode::CREATED, Json(endpoint))),
        Err(e) => {
            tracing::error!("Failed to create endpoint: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

// #[utoipa::path(
//     get,
//     path = "/api/endpoint",
//     responses(
//         (status = 200, description = "List of endpoints", body = Vec<EndpointResponse>),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn list_endpoints(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<EndpointResponse>>, (StatusCode, String)> {
    match app_state.endpoint_service.get_endpoints().await {
        Ok(endpoints) => Ok(Json(endpoints)),
        Err(e) => {
            tracing::error!("Failed to list endpoints: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// List endpoints with pagination, search, and filter support
pub async fn list_endpoints_paginated(
    State(app_state): State<AppState>,
    Query(params): Query<EndpointQueryParams>,
) -> Result<Json<PaginatedEndpointsResponse>, (StatusCode, String)> {
    match app_state
        .endpoint_service
        .get_endpoints_paginated(params.page, params.page_size, params.search, params.status)
        .await
    {
        Ok((endpoints, total)) => {
            let page = params.page.unwrap_or(1);
            let page_size = params.page_size.unwrap_or(10);
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

            let response = PaginatedEndpointsResponse {
                endpoints,
                pagination: PaginationInfo {
                    page,
                    page_size,
                    total,
                    total_pages,
                },
            };

            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to list endpoints with pagination: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

// #[utoipa::path(
//     get,
//     path = "/api/endpoint/{id}",
//     params(
//         ("id" = Uuid, Path, description = "Endpoint ID")
//     ),
//     responses(
//         (status = 200, description = "Endpoint details", body = EndpointDetailResponse),
//         (status = 404, description = "Endpoint not found"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn get_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EndpointDetailResponse>, (StatusCode, String)> {
    match app_state.endpoint_service.get_endpoint_detail(id).await {
        Ok(endpoint) => Ok(Json(endpoint)),
        Err(e) => {
            tracing::error!("Failed to get endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

// #[utoipa::path(
//     put,
//     path = "/api/endpoint/{id}",
//     params(
//         ("id" = Uuid, Path, description = "Endpoint ID")
//     ),
//     request_body = UpdateEndpointRequest,
//     responses(
//         (status = 200, description = "Endpoint updated successfully", body = EndpointResponse),
//         (status = 404, description = "Endpoint not found"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn update_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateEndpointRequest>,
) -> Result<Json<EndpointResponse>, (StatusCode, String)> {
    match app_state
        .endpoint_service
        .update_endpoint(id, request)
        .await
    {
        Ok(endpoint) => Ok(Json(endpoint)),
        Err(e) => {
            tracing::error!("Failed to update endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

// #[utoipa::path(
//     delete,
//     path = "/api/endpoint/{id}",
//     params(
//         ("id" = Uuid, Path, description = "Endpoint ID")
//     ),
//     responses(
//         (status = 204, description = "Endpoint deleted successfully"),
//         (status = 404, description = "Endpoint not found"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn delete_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match app_state.endpoint_service.delete_endpoint(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to delete endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

// #[utoipa::path(
//     get,
//     path = "/api/endpoint/{id}/metrics",
//     params(
//         ("id" = Uuid, Path, description = "Endpoint ID")
//     ),
//     responses(
//         (status = 200, description = "Endpoint metrics", body = EndpointMetrics),
//         (status = 404, description = "Endpoint not found"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn get_endpoint_metrics(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EndpointMetrics>, (StatusCode, String)> {
    match app_state.endpoint_service.get_endpoint_metrics(id).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => {
            tracing::error!("Failed to get metrics for endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

/// Start an endpoint
pub async fn start_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match app_state.endpoint_service.start_endpoint(id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to start endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else if e.to_string().contains("already running") {
                Err((
                    StatusCode::CONFLICT,
                    "Endpoint is already running".to_string(),
                ))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

/// Stop an endpoint
pub async fn stop_endpoint(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match app_state.endpoint_service.stop_endpoint(id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to stop endpoint {}: {}", id, e);
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, "Endpoint not found".to_string()))
            } else if e.to_string().contains("already stopped") {
                Err((
                    StatusCode::CONFLICT,
                    "Endpoint is already stopped".to_string(),
                ))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}
