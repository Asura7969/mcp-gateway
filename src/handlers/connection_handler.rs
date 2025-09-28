use axum::{
    extract::{Query},
    http::StatusCode,
    Json as JsonResponse,
};
use axum::extract::State;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDateTime};
use sqlx::Row;
use crate::state::AppState;
use crate::utils::get_china_time;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionInfo {
    pub id: String,
    pub endpoint_id: String,
    pub session_id: String,
    pub transport_type: i64,
    pub connect_at: DateTime<Utc>,
    pub disconnect_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionCount {
    pub endpoint_id: String,
    pub connect_num: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeSeriesConnectionCount {
    pub time: DateTime<Utc>,
    pub endpoint_id: String,
    pub connect_num: i64,
}

#[derive(Deserialize)]
pub struct ConnectionQueryParams {
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub endpoint_id: Option<String>,
}

/// Get connection logs for a specific endpoint within a time range
pub async fn get_endpoint_connections(
    Query(params): Query<ConnectionQueryParams>,
    State(app_state): State<AppState>,
) -> Result<JsonResponse<Vec<ConnectionInfo>>, (StatusCode, String)> {
    // If endpoint_id is provided in query params, filter by it
    let endpoint_id = params.endpoint_id.clone();
    
    let query_str = if let Some(ref _id) = endpoint_id {
        "SELECT id, endpoint_id, session_id, transport_type, connect_at, disconnect_at 
         FROM endpoint_session_logs 
         WHERE endpoint_id = ?
         ORDER BY connect_at DESC LIMIT 100"
    } else {
        "SELECT id, endpoint_id, session_id, transport_type, connect_at, disconnect_at 
         FROM endpoint_session_logs 
         ORDER BY connect_at DESC LIMIT 100"
    };

    let rows = if let Some(id) = endpoint_id {
        sqlx::query(query_str)
            .bind(id)
            .fetch_all(&app_state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query(query_str)
            .fetch_all(&app_state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let connections: Vec<ConnectionInfo> = rows
        .into_iter()
        .map(|row| {
            let connect_at_naive: NaiveDateTime = row.get("connect_at");
            let disconnect_at_naive: NaiveDateTime = row.get("disconnect_at");
            
            ConnectionInfo {
                id: row.get("id"),
                endpoint_id: row.get("endpoint_id"),
                session_id: row.get("session_id"),
                transport_type: row.get("transport_type"),
                connect_at: DateTime::from_naive_utc_and_offset(connect_at_naive, Utc),
                disconnect_at: DateTime::from_naive_utc_and_offset(disconnect_at_naive, Utc),
            }
        })
        .collect();

    Ok(JsonResponse(connections))
}

/// Get total connection count for a specific endpoint or all endpoints
pub async fn get_endpoint_connection_count(
    Query(params): Query<ConnectionQueryParams>,
    State(app_state): State<AppState>,
) -> Result<JsonResponse<ConnectionCount>, (StatusCode, String)> {

    if let Some(endpoint_id) = params.endpoint_id {
        // Get count for specific endpoint
        let row = sqlx::query(
            "SELECT connect_num FROM endpoint_connection_counts WHERE endpoint_id = ?"
        )
        .bind(endpoint_id.clone())
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let count = if let Some(row) = row {
            row.get::<i64, _>("connect_num")
        } else {
            0
        };

        let result = ConnectionCount {
            endpoint_id: endpoint_id.clone(),
            connect_num: count,
        };
        
        Ok(JsonResponse(result))
    } else {
        // Get total count for all endpoints
        let row = sqlx::query(
            "SELECT COALESCE(SUM(connect_num), 0) as cnt FROM endpoint_connection_counts"
        )
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let total_count: i64 = row.get("cnt");

        // For the "all endpoints" case, we'll use a special identifier
        let result = ConnectionCount {
            endpoint_id: "all".to_string(),
            connect_num: total_count,
        };

        Ok(JsonResponse(result))
    }
}

/// Get time series connection counts for all endpoints within a time range
pub async fn get_time_series_connection_counts(
    Query(_params): Query<ConnectionQueryParams>,
    State(app_state): State<AppState>,
) -> Result<JsonResponse<Vec<TimeSeriesConnectionCount>>, (StatusCode, String)> {
    // For simplicity, we'll return the current connection counts for each endpoint
    // A more complete implementation would aggregate data over time intervals
    
    let rows = sqlx::query(
        "SELECT endpoint_id, connect_num FROM endpoint_connection_counts ORDER BY endpoint_id"
    )
    .fetch_all(&app_state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let counts: Vec<TimeSeriesConnectionCount> = rows
        .into_iter()
        .map(|row| TimeSeriesConnectionCount {
            time: get_china_time(),
            endpoint_id: row.get("endpoint_id"),
            connect_num: row.get("connect_num"),
        })
        .collect();

    Ok(JsonResponse(counts))
}