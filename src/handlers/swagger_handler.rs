use crate::models::{SwaggerToMcpRequest, SwaggerToMcpResponse};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};

// #[utoipa::path(
//     post,
//     path = "/api/swagger",
//     request_body = SwaggerToMcpRequest,
//     responses(
//         (status = 201, description = "Swagger converted to MCP successfully", body = SwaggerToMcpResponse),
//         (status = 400, description = "Bad request - Invalid swagger content"),
//         (status = 500, description = "Internal server error")
//     )
// )]
pub async fn convert_swagger_to_mcp(
    State(app_state): State<AppState>,
    Json(request): Json<SwaggerToMcpRequest>,
) -> Result<(StatusCode, Json<SwaggerToMcpResponse>), (StatusCode, String)> {
    // Validate request
    if request.endpoint_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Endpoint name is required".to_string(),
        ));
    }

    if request.swagger_content.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Swagger content is required".to_string(),
        ));
    }

    match app_state
        .swagger_service
        .convert_swagger_to_mcp(request)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(e) => {
            tracing::error!("Failed to convert swagger to MCP: {}", e);

            // Check if it's a validation error
            let error_msg = e.to_string();
            if error_msg.contains("OpenAPI")
                || error_msg.contains("swagger")
                || error_msg.contains("parse")
            {
                Err((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid swagger content: {}", error_msg),
                ))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, error_msg))
            }
        }
    }
}
