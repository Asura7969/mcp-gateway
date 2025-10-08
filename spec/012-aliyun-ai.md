# 打通阿里云百炼向量模型

如下信息是阿里云百炼向量模型接口信息,需要结合本项目中`embedding.rs`,从`.env`环境配置中加载配置信息，初始化embedding_service

## API 接口信息

```curl
curl --location 'https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings' \
--header "Authorization: Bearer $DASHSCOPE_API_KEY" \
--header 'Content-Type: application/json' \
--data '{
    "model": "text-embedding-v4",
    "input": "风急天高猿啸哀，渚清沙白鸟飞回，无边落木萧萧下，不尽长江滚滚来",  
    "dimensions": "1024",  
    "encoding_format": "float"
}'
```

### 参数说明

* **model**: text-embedding-v4
* **input**: array<string> 或 string 或 file 必选
  * 输入文本的基本信息，可以是字符串，字符串列表， 或者打开的文件（需要Embedding的内容，一行一条）。 
  * 文本限制：
    * 文本数量：
      * 作为字符串时最长支持 2,048 Token。
      > text-embedding-v3或text-embedding-v4模型input是字符串时，最长支持 8,192 Token。
      * 作为字符串列表时最多支持 25 条，每条最长支持 2,048 Token。
      > text-embedding-v3或text-embedding-v4模型input是字符串列表时最多支持 10 条，每条最长支持 8,192 Token。
      * 作为文件时最多支持 25 条，每条最长支持 2,048 Token。
      > text-embedding-v3或text-embedding-v4模型input是文本文件时最多支持 10 行，每行最长支持 8,192 Token。
* **dimensions**: integer 可选
  - 用于用户指定输出向量维度，只适用于text-embedding-v3与text-embedding-v4模型。指定的值只能在2048（仅适用于text-embedding-v4）、1536（仅适用于text-embedding-v4）1024、768、512、256、128或64八个值之间选取，默认值为1024。
* **encoding_format**: string 可选
  - 用于控制返回的Embedding格式，当前仅支持float格式。

### 响应格式

```json
// 成功响应
{
  "data": [
    {
      "embedding": [
        -0.0695386752486229, 0.030681096017360687, ...
      ],
      "index": 0,
      "object": "embedding"
    },
    ...
    {
      "embedding": [
        -0.06348952651023865, 0.060446035116910934, ...
      ],
      "index": 5,
      "object": "embedding"
    }
  ],
  "model": "text-embedding-v3",
  "object": "list",
  "usage": {
    "prompt_tokens": 184,
    "total_tokens": 184
  },
  "id": "73591b79-d194-9bca-8bb5-xxxxxxxxxxxx"
}
```

```json
// 异常响应
{
  "error": {
    "message": "Incorrect API key provided. ",
    "type": "invalid_request_error",
    "param": null,
    "code": "invalid_api_key"
  }
}
```

### 响应字段说明

* **data**: 任务输出信息。(array)
  * **embedding**: 本次调用返回object对象的value，类型是元素为float数据的数组，包含具体Embedding向量。(list)
  * **index**: 本结构中的算法结果对应的输入文字在输入数组中的索引值。(integer)
  * **object**: 本次调用返回的object对象类型，默认为embedding。(string)
* **model**: 本次调用的模型名。(string)
* **object**: 本次调用返回的data类型，默认为list。(string)
* **usage**: object
  * **prompt_tokens**: 用户输入文本转换成Token后的长度。(integer)
  * **total_tokens**: 本次请求输入内容的 Token 数目，算法的计量是根据用户输入字符串被模型Tokenizer解析之后对应的Token数目来进行。(integer)
* **id**: 请求唯一标识。可用于请求明细溯源和问题排查。(string)

## 实现说明

### 1. 配置结构

在 `src/utils/embedding.rs` 中已实现了阿里云百炼的配置结构：

```rust
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
```

### 2. API 调用实现

实现了 `aliyun_embed_text` 方法来调用阿里云百炼 API：

```rust
async fn aliyun_embed_text(&self, text: &str, config: &AliyunBailianConfig) -> Result<Vec<f32>> {
    let request = AliyunEmbeddingRequest {
        model: config.model.clone(),
        input: AliyunEmbeddingInput {
            texts: vec![text.to_string()],
        },
        parameters: Some(AliyunEmbeddingParameters {
            text_type: "document".to_string(),
        }),
    };
    // ... API 调用逻辑
}
```

### 3. 环境配置

在项目根目录的 `.env.example` 文件中添加了阿里云百炼的配置项：

```bash
# 阿里云百炼向量模型配置
ALIYUN_BAILIAN_API_KEY=your_aliyun_bailian_api_key_here
ALIYUN_BAILIAN_MODEL=text-embedding-v1
ALIYUN_BAILIAN_ENDPOINT=https://dashscope.aliyuncs.com/api/v1/services/embeddings/text-embedding/text-embedding
ALIYUN_BAILIAN_WORKSPACE_ID=your_workspace_id_here
EMBEDDING_DIMENSION=1536
EMBEDDING_MODEL_TYPE=aliyun-bailian
```

### 4. 配置加载

在 `src/config/mod.rs` 中实现了从环境变量加载阿里云百炼配置的功能：

```rust
/// 从环境变量创建设置
pub fn from_env() -> Result<Self, ConfigError> {
    // ... 环境变量读取逻辑
    let aliyun_config = if model_type == "aliyun-bailian" {
        if let (Ok(api_key), Ok(model), Ok(endpoint)) = (
            env::var("ALIYUN_BAILIAN_API_KEY"),
            env::var("ALIYUN_BAILIAN_MODEL"),
            env::var("ALIYUN_BAILIAN_ENDPOINT"),
        ) {
            Some(AliyunSettings {
                api_key,
                model,
                endpoint,
                workspace_id: env::var("ALIYUN_BAILIAN_WORKSPACE_ID").ok(),
            })
        } else {
            None
        }
    } else {
        None
    };
    // ...
}
```

## 使用方法

### 1. 配置环境变量

复制 `.env.example` 为 `.env` 并填入你的阿里云百炼 API 密钥：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```bash
ALIYUN_BAILIAN_API_KEY=sk-147da3d9a37545c19010d27dcc5fcdb4
ALIYUN_BAILIAN_MODEL=text-embedding-v4
ALIYUN_BAILIAN_ENDPOINT=https://dashscope.aliyuncs.com/compatible-mode/v1/embeddings
EMBEDDING_DIMENSION=1536
EMBEDDING_MODEL_TYPE=aliyun-bailian
```

### 2. 初始化 EmbeddingService

```rust
use crate::config::Settings;
use crate::utils::embedding::EmbeddingService;

// 从环境变量加载配置
let settings = Settings::from_env()?;
let embedding_config = settings.to_embedding_config();

// 创建 EmbeddingService 实例
let embedding_service = EmbeddingService::new(embedding_config);
```

### 3. 使用向量化服务

```rust
// 对文本进行向量化
let text = "用户管理接口";
let embedding = embedding_service.embed_text(text).await?;

// 对 API 接口进行向量化
let interface_embedding = embedding_service.embed_interface(&api_interface).await?;

// 计算向量相似度
let similarity = embedding_service.cosine_similarity(&embedding1, &embedding2);
```

## 测试

项目中包含了完整的测试用例：

```bash
# 运行 embedding 相关测试
cargo test embedding

# 运行所有测试
cargo test
```

测试包括：
- 基本的向量化功能测试
- 接口向量化测试
- 阿里云百炼配置测试
- 相似度计算测试

## 注意事项

1. **API 密钥安全**: 请确保不要将 API 密钥提交到版本控制系统中
2. **向量维度**: 根据你的需求选择合适的向量维度（64, 128, 256, 512, 768, 1024, 1536, 2048）
3. **文本长度限制**: 注意阿里云百炼对输入文本长度的限制
4. **错误处理**: 实现中包含了完整的错误处理逻辑
5. **性能考虑**: 对于大量文本的向量化，建议使用批量处理

## API 密钥

测试用 API 密钥: `sk-147da3d9a37545c19010d27dcc5fcdb4`

> **警告**: 这是一个示例密钥，请在生产环境中使用你自己的 API 密钥。