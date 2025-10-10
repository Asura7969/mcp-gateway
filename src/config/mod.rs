use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub embedding: Option<EmbeddingSettings>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingSettings {
    pub model_type: String,
    pub dimension: usize,
    pub aliyun: Option<AliyunSettings>,
    pub surrealdb_storage: String,
    pub surrealdb_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AliyunSettings {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
    pub workspace_id: Option<String>,
}

/// 向量化配置
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// 向量维度
    pub dimension: usize,
    /// 模型名称
    pub model_name: String,
    /// API 端点
    pub api_endpoint: Option<String>,
    /// API 密钥
    pub api_key: Option<String>,
    /// 阿里云百炼配置
    pub aliyun_config: Option<AliyunBailianConfig>,
    pub surrealdb_storage: Storage,
    pub surrealdb_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Storage {
    MEMORY,
    ROCKSDB,
}

impl From<String> for Storage {
    fn from(value: String) -> Self {
        if value.eq_ignore_ascii_case("MEMORY") {
            Storage::MEMORY
        } else {
            Storage::ROCKSDB
        }
    }
}

/// 阿里云百炼配置
#[derive(Debug, Clone)]
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

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 384,
            model_name: "simple".to_string(),
            api_endpoint: None,
            api_key: None,
            aliyun_config: None,
            surrealdb_storage: Storage::MEMORY,
            surrealdb_path: None,
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

    /// 转换为 EmbeddingConfig
    pub fn to_embedding_config(&self) -> EmbeddingConfig {
        if let Some(embedding_settings) = &self.embedding {
            let aliyun_config =
                embedding_settings
                    .aliyun
                    .as_ref()
                    .map(|aliyun| AliyunBailianConfig {
                        api_key: aliyun.api_key.clone(),
                        model: aliyun.model.clone(),
                        endpoint: aliyun.endpoint.clone(),
                        workspace_id: aliyun.workspace_id.clone(),
                    });

            EmbeddingConfig {
                dimension: embedding_settings.dimension,
                model_name: embedding_settings.model_type.clone(),
                api_endpoint: None,
                api_key: None,
                aliyun_config,
                surrealdb_storage: embedding_settings.surrealdb_storage.clone().into(),
                surrealdb_path: embedding_settings.surrealdb_path.clone(),
            }
        } else {
            EmbeddingConfig::default()
        }
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
            embedding: Some(EmbeddingSettings {
                model_type: "simple".to_string(),
                dimension: 384,
                aliyun: None,
                surrealdb_storage: "memory".to_string(),
                surrealdb_path: None,
            }),
        }
    }
}
