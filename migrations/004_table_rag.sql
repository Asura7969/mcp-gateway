-- Table RAG: files, datasets, dataset-file mapping, and ingest tasks

-- 文件表
CREATE TABLE IF NOT EXISTS t_file (
    id CHAR(36) PRIMARY KEY,
    type VARCHAR(50) NOT NULL COMMENT '文件类型: csv, excel',
    name VARCHAR(100) DEFAULT NULL COMMENT '文件名称',
    path VARCHAR(200) NOT NULL COMMENT '文件地址: oss路径',
    size BIGINT DEFAULT NULL COMMENT '文件大小',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_type (type),
    INDEX idx_create_time (create_time)
);

-- 知识库表
CREATE TABLE IF NOT EXISTS t_dataset (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL COMMENT '知识库名称',
    description VARCHAR(200) DEFAULT NULL COMMENT '知识库描述',
    type VARCHAR(50) NOT NULL COMMENT '知识库类型: upload(文件上传), remote(远程数据库)',
    table_name VARCHAR(50) NOT NULL COMMENT 'es索引名称(数据表名称)',
    table_schema LONGTEXT NOT NULL COMMENT '数据表schema(json字符串)',
    similarity_threshold DECIMAL(5,2) DEFAULT 0.30 COMMENT '相似度阈值(0.00 - 1.00)',
    max_results INT DEFAULT 10 COMMENT '最大召回数量(1 - 20)',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    UNIQUE KEY unique_dataset_name (name),
    INDEX idx_type (type),
    INDEX idx_table_name (table_name),
    INDEX idx_create_time (create_time)
);

-- 知识库-文件映射表
CREATE TABLE IF NOT EXISTS t_dataset_file (
    id CHAR(36) PRIMARY KEY,
    dataset_id CHAR(36) NOT NULL,
    file_id CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (dataset_id) REFERENCES t_dataset(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES t_file(id) ON DELETE CASCADE,
    UNIQUE KEY unique_dataset_file (dataset_id, file_id),
    INDEX idx_dataset_id (dataset_id),
    INDEX idx_file_id (file_id)
);

-- 任务表
CREATE TABLE IF NOT EXISTS t_task (
    id CHAR(36) PRIMARY KEY,
    dataset_id CHAR(36) NOT NULL,
    file_id CHAR(36) NOT NULL,
    status TINYINT DEFAULT 0 COMMENT '任务状态: 0-已创建;1-处理中;2-完成;3-失败',
    error LONGTEXT DEFAULT NULL COMMENT '任务失败原因',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    FOREIGN KEY (dataset_id) REFERENCES t_dataset(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES t_file(id) ON DELETE CASCADE,
    INDEX idx_dataset_id (dataset_id),
    INDEX idx_file_id (file_id),
    INDEX idx_status (status),
    INDEX idx_create_time (create_time)
);