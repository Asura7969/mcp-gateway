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

    /// 获取模型名称
    pub fn get_model_name(&self) -> &str {
        &self.config.model_name
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
        use crate::config::Settings;
        
        // 使用配置文件中的设置
        let settings = Settings::new().expect("Failed to load settings");
        let embedding_config = settings.to_embedding_config();
        
        let service = EmbeddingService::new(embedding_config);

        // 验证服务创建成功
        assert_eq!(service.get_model_name(), "aliyun");
        
        // 注意：这里不进行实际的 API 调用测试，因为需要真实的网络连接
        // 实际的 API 调用测试应该在集成测试中进行
        println!("✅ 嵌入服务创建成功！");
    }

    #[tokio::test]
    async fn test_aliyun_embedding_service() {
        use crate::config::Settings;
        // 从环境变量加载配置
        let settings = Settings::new().expect("Failed to load settings");

        let config = settings.to_embedding_config();
        let service = EmbeddingService::new(config);

        // 验证配置是否正确加载
        assert!(service.config.aliyun_config.is_some());

        let vec = service.embed_text("测试数据").await.unwrap();
        assert!(vec.len() > 0);
    }
}
