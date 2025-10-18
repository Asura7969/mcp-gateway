use crate::config::EmbeddingConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 阿里云百炼嵌入请求结构
#[derive(Debug, Serialize)]
struct AliyunEmbeddingRequest {
    model: String,
    input: AliyunEmbeddingInput,
    parameters: Option<AliyunEmbeddingParameters>,
    dimensions: Option<usize>,
    encoding_format: Option<String>,
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
    #[allow(dead_code)]
    usage: Option<AliyunUsage>,
    #[allow(dead_code)]
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct AliyunEmbeddingOutput {
    embeddings: Vec<AliyunEmbedding>,
}

#[derive(Debug, Deserialize)]
struct AliyunEmbedding {
    #[allow(dead_code)]
    text_index: i32,
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct AliyunUsage {
    #[allow(dead_code)]
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
        match &self.config.aliyun {
            Some(_) => self.aliyun_embed_text(text).await,
            None => Err(anyhow::anyhow!("Missing config")),
        }
    }

    /// 获取模型名称
    pub fn get_model_name(&self) -> &str {
        &self.config.model_type
    }

    /// 使用阿里云百炼 API 进行文本向量化
    async fn aliyun_embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let config = self
            .config
            .aliyun
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
            dimensions: Some(self.config.dimension),
            encoding_format: Some("float".to_string()),
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

        // 添加调试日志，打印返回的向量信息
        let embedding = &api_response.output.embeddings[0].embedding;
        tracing::debug!(
            "阿里云百炼 API 返回向量数据长度: {:?}",
            &api_response.output.embeddings.len()
        );
        Ok(embedding.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use tracing::warn;

    #[tokio::test]
    async fn test_embedding_service_creation() {
        use crate::config::Settings;

        // 使用配置文件中的设置
        let settings = Settings::new().expect("Failed to load settings");
        let embedding_config = settings.embedding;

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

        let config = settings.embedding;
        let service = EmbeddingService::new(config);

        // 验证配置是否正确加载
        assert!(service.config.aliyun.is_some());

        let vec = service.embed_text("测试数据").await.unwrap();
        assert!(vec.len() > 0);
    }

    #[tokio::test]
    async fn test_aliyun_embed_text_dimension_consistency() {
        let settings = Settings::new().unwrap_or_else(|_| {
            warn!("Failed to load configuration, using defaults");
            Settings::default()
        });

        let embedding_config = settings.embedding;
        warn!("embedding_config: {:?}", embedding_config);

        let service = EmbeddingService::new(embedding_config);

        // 测试不同长度的文本内容
        let test_texts = vec![
            // 短文本 (约10个字符)
            "短文本",
            // 中等长度文本 (约50个字符)
            "这是一个中等长度的测试文本，用于验证向量化服务的稳定性和一致性。",
            // 长文本 (约200个字符)
            "这是一个相对较长的测试文本，包含了更多的信息和内容。我们使用这个文本来测试阿里云百炼向量化服务在处理不同长度文本时的表现。这个测试的目的是验证无论输入文本的长度如何变化，返回的向量维度都应该保持一致，即1024维。这对于确保向量化服务的稳定性和可靠性非常重要。",
            // 超长文本 (约500个字符)
            "这是一个超长的测试文本，用于验证向量化服务在处理大量文本内容时的性能和稳定性。在实际应用中，我们经常需要处理各种长度的文本，从简短的标题到详细的描述，再到完整的文档内容。向量化服务必须能够稳定地处理这些不同长度的输入，并始终返回相同维度的向量表示。这种一致性对于后续的相似度计算、聚类分析和检索任务都至关重要。通过这个测试，我们可以确保我们的向量化服务在各种使用场景下都能提供可靠的结果。无论是处理用户查询、API接口描述，还是其他类型的文本数据，服务都应该表现出色。",
            // 包含特殊字符的文本
            "测试文本！@#$%^&*()_+-={}[]|\\:;\"'<>,.?/~`包含各种特殊字符和标点符号，用于验证向量化服务的鲁棒性。",
            // 英文文本
            "This is an English text to test the embedding service with different languages and character sets.",
            // 混合语言文本
            "这是一个中英文混合的测试文本 with mixed Chinese and English content to verify the robustness of the embedding service.",
        ];

        println!("🚀 开始测试不同长度文本的 embedding 维度一致性...");

        let mut all_embeddings = Vec::new();

        for (i, text) in test_texts.iter().enumerate() {
            println!(
                "📝 测试文本 {} (长度: {} 字符): {}",
                i + 1,
                text.chars().count(),
                if text.len() > 50 {
                    format!("{}...", &text.chars().take(50).collect::<String>())
                } else {
                    text.to_string()
                }
            );

            match service.aliyun_embed_text(text).await {
                Ok(embedding) => {
                    println!("✅ 成功获取 embedding，维度: {}", embedding.len());

                    // 验证 embedding 长度是否为 1024
                    assert_eq!(
                        embedding.len(),
                        1024,
                        "文本 '{}' 的 embedding 维度应该是 1024，但实际是 {}",
                        text,
                        embedding.len()
                    );

                    // 验证 embedding 不全为零
                    assert!(
                        embedding.iter().any(|&x| x != 0.0),
                        "文本 '{}' 的 embedding 不应该全为零",
                        text
                    );

                    all_embeddings.push(embedding);
                }
                Err(e) => {
                    println!("❌ 获取 embedding 失败: {}", e);
                    panic!("文本 '{}' 的向量化失败: {}", text, e);
                }
            }
        }

        // 验证所有 embedding 的维度都相同
        let expected_dimension = 1024;
        for (i, embedding) in all_embeddings.iter().enumerate() {
            assert_eq!(
                embedding.len(),
                expected_dimension,
                "第 {} 个文本的 embedding 维度不一致，期望: {}，实际: {}",
                i + 1,
                expected_dimension,
                embedding.len()
            );
        }

        // 验证不同文本的 embedding 确实不同（避免返回相同的向量）
        if all_embeddings.len() >= 2 {
            let first_embedding = &all_embeddings[0];
            let second_embedding = &all_embeddings[1];

            // 计算余弦相似度，确保不同文本的向量不完全相同
            let dot_product: f32 = first_embedding
                .iter()
                .zip(second_embedding.iter())
                .map(|(a, b)| a * b)
                .sum();

            let norm_a: f32 = first_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = second_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            let cosine_similarity = dot_product / (norm_a * norm_b);

            println!("📊 前两个文本的余弦相似度: {:.4}", cosine_similarity);

            // 确保相似度不是 1.0（即不完全相同）
            assert!(
                cosine_similarity < 0.99,
                "不同文本的 embedding 不应该完全相同，相似度: {}",
                cosine_similarity
            );
        }

        println!("🎉 所有测试通过！embedding 维度一致性验证成功。");
        println!("📈 测试统计:");
        println!("   - 测试文本数量: {}", test_texts.len());
        println!("   - 期望维度: {}", expected_dimension);
        println!("   - 所有 embedding 维度均为: {}", expected_dimension);
    }
}
