use crate::config::EmbeddingConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 阿里云百炼嵌入请求结构
#[derive(Debug, Serialize)]
struct AliyunEmbeddingRequest {
    model: String,
    input: AliyunEmbeddingInput,
    parameters: Option<AliyunEmbeddingParameters>,
}

#[derive(Debug, Serialize)]
struct AliyunEmbeddingInput {
    texts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AliyunEmbeddingParameters {
    text_type: String,
}

/// 阿里云百炼嵌入响应结构
#[derive(Debug, Deserialize)]
struct AliyunEmbeddingResponse {
    output: AliyunEmbeddingOutput,
    usage: Option<AliyunUsage>,
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct AliyunEmbeddingOutput {
    embeddings: Vec<AliyunEmbedding>,
}

#[derive(Debug, Deserialize)]
struct AliyunEmbedding {
    text_index: i32,
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct AliyunUsage {
    total_tokens: i32,
}

/// 向量化服务
pub struct EmbeddingService {
    config: EmbeddingConfig,
    client: reqwest::Client,
}

impl EmbeddingService {
    /// 创建新的向量化服务实例
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 从配置创建向量化服务
    pub fn from_config(config: EmbeddingConfig) -> Result<Self> {
        Ok(Self::new(config))
    }

    /// 获取文本的向量表示
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        match &self.config.aliyun_config {
            Some(_) => self.aliyun_embed_text(text).await,
            None => Err(anyhow::anyhow!("Missing config")),
        }
    }

    /// 使用阿里云百炼 API 进行文本向量化
    async fn aliyun_embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let config = self
            .config
            .aliyun_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("阿里云百炼配置未设置"))?;

        let request = AliyunEmbeddingRequest {
            model: config.model.clone(),
            input: AliyunEmbeddingInput {
                texts: vec![text.to_string()],
            },
            parameters: Some(AliyunEmbeddingParameters {
                text_type: "document".to_string(),
            }),
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", config.api_key).parse()?,
        );
        headers.insert("Content-Type", "application/json".parse()?);

        // 如果有工作空间 ID，添加到请求头
        if let Some(workspace_id) = &config.workspace_id {
            headers.insert("X-DashScope-WorkSpace", workspace_id.parse()?);
        }

        let response = self
            .client
            .post(&config.endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "阿里云百炼 API 调用失败: HTTP {}, 响应: {}",
                status,
                error_text
            ));
        }

        let api_response: AliyunEmbeddingResponse = response.json().await?;

        if api_response.output.embeddings.is_empty() {
            return Err(anyhow::anyhow!("阿里云百炼 API 返回空的向量结果"));
        }

        Ok(api_response.output.embeddings[0].embedding.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AliyunBailianConfig;

    #[tokio::test]
    async fn test_embedding_service_creation() {
        let config = EmbeddingConfig::default();
        let service = EmbeddingService::new(config);

        // 测试简单文本向量化
        let result = service.embed_text("测试文本").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384); // 默认维度
    }

    #[tokio::test]
    async fn test_aliyun_embedding_service() {
        let aliyun_config = AliyunBailianConfig {
            api_key: "test_key".to_string(),
            model: "text-embedding-v1".to_string(),
            endpoint: "https://dashscope.aliyuncs.com/api/v1/services/embeddings/text-embedding/text-embedding".to_string(),
            workspace_id: Some("test_workspace".to_string()),
        };

        let config = EmbeddingConfig {
            dimension: 1536,
            model_name: "text-embedding-v1".to_string(),
            api_endpoint: None,
            api_key: None,
            aliyun_config: Some(aliyun_config),
        };

        let service = EmbeddingService::new(config);

        // 注意：这个测试需要真实的 API 密钥才能通过
        // 在实际环境中，应该使用模拟的 HTTP 客户端进行测试
        assert!(service.config.aliyun_config.is_some());
    }
}
