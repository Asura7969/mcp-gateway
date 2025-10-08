use anyhow::Result;
use std::collections::HashMap;

/// 向量化服务配置
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// 向量维度
    pub dimension: usize,
    /// 模型名称
    pub model_name: String,
    /// API端点（如果使用外部服务）
    pub api_endpoint: Option<String>,
    /// API密钥
    pub api_key: Option<String>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            dimension: 384, // 使用较小的维度以提高性能
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            api_endpoint: None,
            api_key: None,
        }
    }
}

/// 向量化服务
pub struct EmbeddingService {
    config: EmbeddingConfig,
}

impl EmbeddingService {
    /// 创建新的向量化服务
    pub fn new(config: EmbeddingConfig) -> Self {
        Self { config }
    }

    /// 创建默认的向量化服务
    pub fn default() -> Self {
        Self::new(EmbeddingConfig::default())
    }

    /// 将文本转换为向量
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        // 这里实现一个简化的向量化逻辑
        // 在实际应用中，你可能需要调用外部API或使用本地模型
        self.simple_text_embedding(text)
    }

    /// 为API接口生成向量嵌入
    pub async fn embed_interface(&self, interface: &crate::models::interface_relation::ApiInterface) -> Result<Vec<f32>> {
        // 组合接口的各种文本信息
        let combined_text = self.combine_interface_text(interface);
        self.embed_text(&combined_text).await
    }

    /// 计算两个向量的余弦相似度
    pub fn cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1 * norm2)
    }

    /// 组合接口的文本信息用于向量化
    fn combine_interface_text(&self, interface: &crate::models::interface_relation::ApiInterface) -> String {
        let mut text_parts = Vec::new();

        // 添加接口摘要（替代name字段）
        if let Some(summary) = &interface.summary {
            text_parts.push(summary.clone());
        }

        // 添加接口路径
        text_parts.push(format!("{} {}", interface.method, interface.path));

        // 添加描述
        if let Some(description) = &interface.description {
            text_parts.push(description.clone());
        }

        // 添加服务描述
        if let Some(service_desc) = &interface.service_description {
            text_parts.push(service_desc.clone());
        }

        // 添加标签
        if !interface.tags.is_empty() {
            text_parts.push(interface.tags.join(" "));
        }

        // 添加业务领域
        if let Some(domain) = &interface.domain {
            text_parts.push(domain.clone());
        }

        // 添加路径参数信息
        for param in &interface.path_params {
            text_parts.push(format!("{} {}", param.name, param.param_type));
            if let Some(desc) = &param.description {
                text_parts.push(desc.clone());
            }
        }

        // 添加查询参数信息
        for param in &interface.query_params {
            text_parts.push(format!("{} {}", param.name, param.param_type));
            if let Some(desc) = &param.description {
                text_parts.push(desc.clone());
            }
        }

        // 添加请求头参数信息
        for param in &interface.header_params {
            text_parts.push(format!("{} {}", param.name, param.param_type));
            if let Some(desc) = &param.description {
                text_parts.push(desc.clone());
            }
        }

        // 添加请求体参数信息
        for param in &interface.body_params {
            text_parts.push(format!("{} {}", param.name, param.param_type));
            if let Some(desc) = &param.description {
                text_parts.push(desc.clone());
            }
        }

        text_parts.join(" ")
    }

    /// 简化的文本向量化实现（用于演示）
    /// 在生产环境中，应该使用真正的embedding模型
    fn simple_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let text_lower = text.to_lowercase();
        let mut embedding = vec![0.0; self.config.dimension];

        // 使用简单的哈希和字符统计来生成向量
        // 这只是一个演示实现，实际应用中需要使用真正的embedding模型
        
        // 基于字符频率
        let mut char_counts: HashMap<char, f32> = HashMap::new();
        for ch in text_lower.chars() {
            if ch.is_alphanumeric() {
                *char_counts.entry(ch).or_insert(0.0) += 1.0;
            }
        }

        // 将字符频率映射到向量维度
        for (i, (_ch, count)) in char_counts.iter().enumerate() {
            if i < self.config.dimension {
                embedding[i] = *count / text.len() as f32;
            }
        }

        // 基于单词长度分布
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            let idx = (word.len() + i * 7) % self.config.dimension;
            embedding[idx] += 0.1;
        }

        // 基于文本长度
        let length_factor = (text.len() as f32).ln() / 10.0;
        for i in 0..std::cmp::min(10, self.config.dimension) {
            embedding[i] += length_factor;
        }

        // 归一化向量
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_service() {
        let service = EmbeddingService::default();
        
        let text = "用户管理接口";
        let embedding = service.embed_text(text).await.unwrap();
        
        assert_eq!(embedding.len(), 384);
        
        // 测试相似度计算
        let text2 = "用户管理API";
        let embedding2 = service.embed_text(text2).await.unwrap();
        
        let similarity = service.cosine_similarity(&embedding, &embedding2);
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
    }

    #[tokio::test]
    async fn test_interface_embedding() {
        use crate::models::interface_relation::*;
        
        let service = EmbeddingService::default();
        
        let interface = ApiInterface {
            path: "/api/users".to_string(),
            method: "GET".to_string(),
            summary: Some("获取用户列表".to_string()),
            description: Some("获取系统中所有用户的列表".to_string()),
            operation_id: Some("getUsers".to_string()),
            path_params: vec![],
            query_params: vec![],
            header_params: vec![],
            body_params: vec![],
            request_schema: None,
            response_schema: None,
            tags: vec!["用户".to_string(), "管理".to_string()],
            domain: Some("用户管理".to_string()),
            deprecated: false,
            service_description: Some("用户管理服务".to_string()),
            embedding: None,
            embedding_model: None,
            embedding_updated_at: None,
        };
        
        let embedding = service.embed_interface(&interface).await.unwrap();
        assert_eq!(embedding.len(), 384);
    }
}