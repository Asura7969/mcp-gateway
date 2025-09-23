-- Create endpoints table
CREATE TABLE IF NOT EXISTS endpoints (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    swagger_content LONGTEXT NOT NULL,
    status ENUM('running', 'stopped', 'deleted') NOT NULL DEFAULT 'stopped',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    connection_count INT DEFAULT 0,
    INDEX idx_name (name),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);

-- Create endpoint_metrics table
CREATE TABLE IF NOT EXISTS endpoint_metrics (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    request_count BIGINT UNSIGNED DEFAULT 0,
    response_count BIGINT UNSIGNED DEFAULT 0,
    error_count BIGINT UNSIGNED DEFAULT 0,
    avg_response_time DECIMAL(10,3) DEFAULT 0.000,
    current_connections INT DEFAULT 0,
    total_connection_time BIGINT UNSIGNED DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE,
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_created_at (created_at)
);

-- Create endpoint_logs table for request/response logging
CREATE TABLE IF NOT EXISTS endpoint_logs (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    request_id CHAR(36) NOT NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(500) NOT NULL,
    status_code INT,
    response_time_ms INT,
    request_body TEXT,
    response_body TEXT,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE,
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_request_id (request_id),
    INDEX idx_created_at (created_at),
    INDEX idx_status_code (status_code)
);