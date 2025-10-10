use crate::models::endpoint::ApiDetail;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Datetime;
use surrealdb::RecordId;
use utoipa::ToSchema;

/// 接口节点 - 表示一个API接口，基于ApiDetail结构设计
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ApiInterface {
    /// 接口路径，如 /api/users/{id}
    pub path: String,
    /// HTTP方法，如 GET, POST, PUT, DELETE
    pub method: String,
    /// 接口摘要/标题
    pub summary: Option<String>,
    /// 接口详细描述
    pub description: Option<String>,
    /// 操作ID，用于唯一标识接口
    pub operation_id: Option<String>,
    /// 路径参数
    pub path_params: Vec<ApiParameter>,
    /// 查询参数
    pub query_params: Vec<ApiParameter>,
    /// 请求头参数
    pub header_params: Vec<ApiParameter>,
    /// 请求体参数
    pub body_params: Vec<ApiParameter>,
    /// 请求体schema
    pub request_schema: Option<String>,
    /// 响应schema
    pub response_schema: Option<String>,
    /// 接口标签/分类
    pub tags: Vec<String>,
    /// 业务领域，如 user, order, product
    pub domain: Option<String>,
    /// 是否已弃用
    pub deprecated: bool,
    /// 服务描述（来自swagger spec）
    pub service_description: Option<String>,
    /// 语义向量嵌入（用于向量搜索）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    /// 向量嵌入的版本/模型标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    /// 向量嵌入生成时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_updated_at: Option<String>,
}

/// API参数定义，基于ApiDetail中的参数结构
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ApiParameter {
    /// 参数名称
    pub name: String,
    /// 参数类型：string, integer, boolean, object, array等
    pub param_type: String,
    /// 是否必需
    pub required: bool,
    /// 参数描述
    pub description: Option<String>,
    /// 示例值
    pub example: Option<String>,
    /// 默认值
    pub default_value: Option<String>,
    /// 枚举值
    pub enum_values: Option<Vec<String>>,
    /// 格式信息（如date, email等）
    pub format: Option<String>,
}

impl From<ApiDetail> for ApiInterface {
    fn from(api_detail: ApiDetail) -> Self {
        Self {
            path: api_detail.path,
            method: api_detail.method,
            summary: api_detail.summary,
            description: api_detail.description,
            operation_id: api_detail.operation_id,
            path_params: api_detail
                .path_params
                .into_iter()
                .map(|p| p.into())
                .collect(),
            query_params: api_detail
                .query_params
                .into_iter()
                .map(|p| p.into())
                .collect(),
            header_params: api_detail
                .header_params
                .into_iter()
                .map(|p| p.into())
                .collect(),
            body_params: Vec::new(), // endpoint::ApiDetail没有body_params字段
            request_schema: api_detail
                .request_body_schema
                .map(|s| serde_json::to_string(&s).unwrap_or_default()),
            response_schema: Some(
                serde_json::to_string(&api_detail.response_schema).unwrap_or_default(),
            ),
            tags: Vec::new(), // 需要从swagger spec中提取
            domain: None,
            deprecated: false,         // 需要从swagger spec中提取
            service_description: None, // 需要从swagger spec中提取
            embedding: None,
            embedding_model: None,
            embedding_updated_at: None,
        }
    }
}

impl From<crate::models::endpoint::ApiParameter> for ApiParameter {
    fn from(param: crate::models::endpoint::ApiParameter) -> Self {
        Self {
            name: param.name,
            param_type: param.param_type,
            required: param.required,
            description: param.description,
            example: None,       // endpoint::ApiParameter没有example字段
            default_value: None, // endpoint::ApiParameter没有default_value字段
            enum_values: None,   // endpoint::ApiParameter没有enum_values字段
            format: None,        // endpoint::ApiParameter没有format字段
        }
    }
}

/// 带评分的接口结果
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InterfaceWithScore {
    /// 接口信息
    pub interface: ApiInterface,
    /// 匹配评分 (0.0-1.0)
    pub score: f64,
    /// 匹配原因说明
    pub match_reason: String,
    /// 向量相似度分数（如果使用了向量搜索）
    pub similarity_score: Option<f32>,
    /// 搜索类型：keyword, vector, hybrid
    pub search_type: String,
}

/// SurrealDB中的接口记录
#[derive(Debug, Serialize, Deserialize)]
pub struct InterfaceRecord {
    pub id: Option<RecordId>,
    #[serde(flatten)]
    pub interface: ApiInterface,
    pub project_id: String,
    pub version: Option<String>,
    pub score: Option<f32>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// 错误类型
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InterfaceRelationError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
}

/// Swagger JSON解析请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SwaggerParseRequest {
    /// Swagger JSON数据
    pub swagger_json: serde_json::Value,
    /// 项目ID
    pub project_id: String,
    /// 版本信息（可选）
    pub version: Option<String>,
    /// 是否生成向量嵌入
    pub generate_embeddings: Option<bool>,
}

/// Swagger解析响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SwaggerParseResponse {
    /// 解析成功的接口数量
    pub parsed_interfaces_count: u32,
    /// 存储的接口数量
    pub stored_interfaces_count: u32,
    /// 生成的依赖关系数量
    pub dependencies_count: u32,
    /// 处理耗时（毫秒）
    pub processing_time_ms: u64,
    /// 错误信息（如果有）
    pub errors: Vec<String>,
    /// 警告信息（如果有）
    pub warnings: Vec<String>,
}

/// 接口检索请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InterfaceSearchRequest {
    /// 搜索关键词或查询文本
    pub query: String,
    /// 项目ID（可选，用于限制查询范围）
    pub project_id: Option<String>,
    /// 最大返回接口数量
    pub max_results: Option<u32>,
    /// 是否启用向量搜索（语义搜索）
    pub enable_vector_search: Option<bool>,
    /// 是否启用关键词搜索
    pub enable_keyword_search: Option<bool>,
    /// 向量搜索权重（0.0-1.0），与关键词搜索结果混合
    pub vector_search_weight: Option<f32>,
    /// 向量搜索相似度阈值（0.0-1.0）
    pub similarity_threshold: Option<f32>,
    /// 搜索模式：keyword, vector, hybrid
    pub search_mode: Option<String>,
    /// 过滤条件
    pub filters: Option<InterfaceSearchFilters>,
}

/// 接口搜索过滤条件
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InterfaceSearchFilters {
    /// HTTP方法过滤
    pub methods: Option<Vec<String>>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 域名过滤
    pub domain: Option<String>,
    /// 是否包含已废弃的接口
    pub include_deprecated: Option<bool>,
    /// 路径前缀过滤
    pub path_prefix: Option<String>,
}

/// 接口检索响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InterfaceSearchResponse {
    /// 匹配的接口列表
    pub interfaces: Vec<InterfaceWithScore>,
    /// 查询耗时（毫秒）
    pub query_time_ms: u64,
    /// 总匹配数量
    pub total_count: u32,
    /// 搜索模式
    pub search_mode: String,
}
