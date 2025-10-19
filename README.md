# MCP Gateway

一个智能的 Model Context Protocol (MCP) 网关系统，将 Swagger API 转换为 MCP 工具，并提供强大的向量检索和 RAG 功能。

## 🎯 核心功能

### 1. MCP 协议转换
- **Swagger 转 MCP**：自动将 Swagger/OpenAPI 规范转换为 MCP 工具
- **多协议支持**：支持 stdio、SSE 和 Streamable HTTP 三种传输协议
- **会话管理**：实时统计和管理 MCP 会话连接数

### 2. 智能向量检索
- **多向量数据库**：支持 Elasticsearch 和 PgVector，可通过配置动态切换
- **混合检索**：结合向量搜索和全文检索，提供更精准的结果
- **阿里云百炼集成**：使用阿里云百炼模型生成高质量向量嵌入
- **自动同步**：端点保存/修改时自动更新向量数据

### 3. RAG 检索系统
- **接口语义搜索**：基于自然语言查询相关 API 接口
- **Chunk 级检索**：支持细粒度的接口详情查询
- **前端调试界面**：提供可视化的向量检索调试工具

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Node.js 18+
- MySQL 8.0+
- Elasticsearch 8.x 或 PostgreSQL 15+ (可选)

### 启动服务

```bash
# 1. 克隆项目
git clone <repository-url>
cd mcp-gateway

# 2. 配置环境变量
cp config/default.toml.example config/default.toml
# 编辑配置文件，设置数据库和向量数据库连接

# 3. 启动服务
./scripts/deployment/start.sh
```

### 使用示例

```bash
# 1. 转换 Swagger 为 MCP 工具
curl -X POST http://localhost:3000/api/swagger \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint_name": "user-api",
    "description": "用户管理API",
    "swagger_content": "{...}"
  }'

# 2. 向量检索接口
curl -X POST http://localhost:3000/api/interface-retrieval/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "获取用户信息",
    "search_type": "Hybrid",
    "max_results": 10
  }'
```

## 🏗️ 技术架构

### 后端技术栈
- **Rust** - 高性能系统编程语言
- **Axum** - 现代异步 Web 框架
- **SQLx** - 类型安全的数据库访问
- **Elasticsearch/PgVector** - 向量数据库
- **RMCP** - Rust MCP 协议实现

### 前端技术栈
- **React 18 + TypeScript** - 现代前端框架
- **Tailwind CSS + Shadcn UI** - 美观的用户界面
- **Vite** - 快速构建工具

### 核心组件
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Swagger API   │───▶│   MCP Gateway   │───▶│   MCP Client    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Vector Search  │
                    │ (ES/PgVector)   │
                    └─────────────────┘
```

## ⚙️ 配置说明

### 核心配置 (config/default.toml)

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "mysql://user:password@localhost:3306/mcp_gateway"

[embedding]
# 向量数据库类型: "elasticsearch" 或 "pgvectorrs"
vector_type = "elasticsearch"

# 阿里云百炼配置
provider = "dashscope"
api_key = "your-api-key"
model = "text-embedding-v2"

[elasticsearch]
url = "http://localhost:9200"

[pgvector]
url = "postgresql://user:password@localhost:5432/vector_db"
```

### 环境变量

```bash
# 数据库
DATABASE_URL=mysql://user:password@localhost:3306/mcp_gateway

# 阿里云百炼
DASHSCOPE_API_KEY=your-api-key

# Elasticsearch (可选)
ELASTICSEARCH_URL=http://localhost:9200

# PgVector (可选)
PGVECTOR_URL=postgresql://user:password@localhost:5432/vector_db
```

## 🔧 开发指南

### 项目结构
```
mcp-gateway/
├── src/
│   ├── handlers/          # API 处理器
│   ├── services/          # 业务逻辑
│   │   ├── swagger_service.rs      # Swagger 转换
│   │   ├── elastic_search.rs       # Elasticsearch 集成
│   │   ├── pgvectorrs_search.rs    # PgVector 集成
│   │   └── interface_retrieval_service.rs  # 接口检索
│   ├── models/            # 数据模型
│   └── utils/             # 工具函数
├── web/                   # React 前端
└── config/                # 配置文件
```

### 运行测试
```bash
# 后端测试
cargo test

# 前端测试
cd web && npm test
```

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

**项目状态**: ✅ 生产就绪  
**维护者**: MCP Gateway Team  
**最后更新**: 2025年10月