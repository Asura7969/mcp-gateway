use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub mcp_call_max_connections: u32,
}

/// 向量化配置
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingConfig {
    /// 模型类型
    pub model_type: String,
    /// 向量维度
    pub dimension: usize,
    /// 向量存储类型
    pub vector_type: VectorType,
    /// 阿里云百炼配置
    pub aliyun: Option<AliyunBailianConfig>,
    /// PgVector-RS配置
    pub pgvectorrs: Option<PgvectorRsConfig>,
    /// SurrealDB配置
    pub elasticsearch: Option<ElasticsearchConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorType {
    Elasticsearch,
    PgVectorRs,
}

impl From<String> for VectorType {
    fn from(value: String) -> Self {
        if value.to_lowercase().eq("elasticsearch") {
            VectorType::Elasticsearch
        } else {
            VectorType::PgVectorRs
        }
    }
}

/// elasticsearch配置
#[derive(Debug, Clone, Deserialize)]
pub struct ElasticsearchConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
}

/// 阿里云百炼配置
#[derive(Debug, Clone, Deserialize)]
pub struct AliyunBailianConfig {
    /// API Key
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// API 端点
    pub endpoint: String,
    /// 工作空间 ID
    pub workspace_id: Option<String>,
}

/// PgVector-RS配置
#[derive(Debug, Clone, Deserialize)]
pub struct PgvectorRsConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_type: "simple".to_string(),
            dimension: 1024,
            vector_type: VectorType::PgVectorRs,
            aliyun: None,
            pgvectorrs: None,
            elasticsearch: None,
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            .build()?;

        s.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "mysql://mcpuser:mcppassword@localhost:3306/mcp_gateway".to_string(),
                max_connections: 5,
                mcp_call_max_connections: 2,
            },
            embedding: EmbeddingConfig {
                model_type: "simple".to_string(),
                dimension: 1024,
                vector_type: VectorType::Elasticsearch,
                aliyun: None,
                pgvectorrs: Some(PgvectorRsConfig {
                    database: "mcp".to_string(),
                    user: "postgres".to_string(),
                    password: "mcp123456".to_string(),
                    host: "localhost".to_string(),
                    port: "5432".to_string(),
                }),
                elasticsearch: None,
            },
        }
    }
}
