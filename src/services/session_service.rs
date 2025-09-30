use crate::models::DbPool;
use crate::utils::get_china_time;
use dashmap::DashMap;
use rmcp::transport::sse_server::{EndpointId, McpType};
use rmcp::transport::streamable_http_server::SessionId;
use sqlx::Row;
use uuid::Uuid;

///
pub struct SessionService {
    pool: DbPool,
    cache: DashMap<SessionId, Status>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum Status {
    Init,
    Created,
    Destroy,
}

impl SessionService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            cache: Default::default(),
        }
    }

    /// 此方法值针对streamable做缓存
    pub fn pre_save_cache(&self, session_id: SessionId) {
        match self.cache.get(&session_id) {
            Some(_) => {}
            None => {
                self.cache.insert(session_id, Status::Init);
            }
        }
    }

    /// 此方法值针对streamable做缓存
    pub async fn destroy_session(&self, session_id: &SessionId) {
        if self.eq_status(session_id, &[Status::Created, Status::Init]) {
            self.cache.alter(session_id, |_, _v| Status::Destroy);
            self.remove_session("".to_string(), session_id.clone(), McpType::STREAMABLE)
                .await
        }
    }

    fn eq_status(&self, session_id: &SessionId, other: &[Status]) -> bool {
        match self.cache.get(session_id) {
            Some(status) => other.contains(status.value()),
            None => false,
        }
    }

    pub async fn add_session(
        &self,
        endpoint_id: EndpointId,
        session_id: SessionId,
        mcp_type: McpType,
    ) {
        if matches!(mcp_type, McpType::STREAMABLE)
            && self.eq_status(&session_id, &[Status::Created, Status::Destroy])
        {
            return;
        }
        let now = get_china_time();
        let id = Uuid::new_v4();

        let mcp_type_code = match mcp_type {
            McpType::SSE => 1,
            McpType::STREAMABLE => 2,
        };

        // Insert session log
        if let Err(e) = sqlx::query(
            r#"
                INSERT INTO endpoint_session_logs (id, endpoint_id, session_id, transport_type, connect_at, disconnect_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
        )
            .bind(id.to_string())
            .bind(&endpoint_id)
            .bind(session_id.to_string())
            .bind(mcp_type_code)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
        {
            tracing::error!("Failed to insert endpoint session log: {}", e);
        }

        // Update connection count
        if let Err(e) = sqlx::query("INSERT INTO endpoint_connection_counts (id, endpoint_id, connect_num) VALUES (?, ?, 1) ON DUPLICATE KEY UPDATE connect_num = connect_num + 1")
            .bind(Uuid::new_v4().to_string())
            .bind(&endpoint_id)
            .execute(&self.pool)
            .await
        {
            tracing::error!("Failed to update connection count for endpoint {}: {}", endpoint_id, e);
        }

        if matches!(mcp_type, McpType::STREAMABLE) {
            self.cache.alter(&session_id, |_, _v| Status::Created);
        }
    }

    pub async fn remove_session(
        &self,
        endpoint_id: EndpointId,
        session_id: SessionId,
        mcp_type: McpType,
    ) {
        let endpoint_id = match mcp_type {
            McpType::SSE => endpoint_id,
            McpType::STREAMABLE => {
                let row = sqlx::query(
                    "SELECT endpoint_id FROM endpoint_session_logs WHERE session_id = ?",
                )
                .bind(session_id.to_string())
                .fetch_one(&self.pool)
                .await
                .unwrap();
                row.get("endpoint_id")
            }
        };

        if let Err(e) = sqlx::query("UPDATE endpoint_session_logs SET disconnect_at = ? WHERE endpoint_id = ? and session_id = ?")
            .bind(get_china_time())
            .bind(&endpoint_id)
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await {
            tracing::error!("Failed to update endpoint session log: {}", e);
        }

        if let Err(e) = sqlx::query("INSERT INTO endpoint_connection_counts (id, endpoint_id, connect_num) VALUES (?, ?, 0) ON DUPLICATE KEY UPDATE connect_num = GREATEST(0, connect_num - 1)")
            .bind(Uuid::new_v4().to_string())
            .bind(&endpoint_id)
            .execute(&self.pool)
            .await {
            tracing::error!("Failed to update connection count for endpoint {}: {}", endpoint_id, e);
        }
    }
}
