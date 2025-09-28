-- endpoint_id, session_id映射表，endpoint_id & session_id为唯一索引
-- 新请求创建时，connect_at与disconnect_at时间一致，连接断开后，修改disconnect_at时间
CREATE TABLE IF NOT EXISTS endpoint_session_logs (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    session_id CHAR(36) NOT NULL,
    transport_type SMALLINT NOT NULL COMMENT '1:sse, 2:streamable',
    connect_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    disconnect_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_session_id (session_id),
    INDEX idx_connect_at (connect_at),
    INDEX idx_disconnect_at (disconnect_at),
    UNIQUE KEY unique_endpoint_session (endpoint_id, session_id)
);

-- endpoint连接数量表(此表主要针对实时查询，当前时间各个endpoint的连接数量)
CREATE TABLE IF NOT EXISTS endpoint_connection_counts (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL UNIQUE,
    connect_num BIGINT DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_endpoint_id (endpoint_id)
);