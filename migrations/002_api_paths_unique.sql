-- Create api_paths table to store unique API paths and methods for each endpoint
CREATE TABLE IF NOT EXISTS api_paths (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    path VARCHAR(500) NOT NULL,
    method VARCHAR(10) NOT NULL,
    operation_id VARCHAR(255),
    summary TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE,
    UNIQUE KEY unique_endpoint_path_method (endpoint_id, path, method),
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_path (path),
    INDEX idx_method (method)
);

-- Note: Triggers will be implemented in Rust code since MySQL doesn't have good JSON parsing
-- We'll handle this in the application layer
