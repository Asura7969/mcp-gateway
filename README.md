# MCP Gateway 项目

一个基于 Rust 和 React 的 Model Context Protocol (MCP) 网关系统。

## 🚀 快速开始

### 环境要求

- **Rust** (1.70+)
- **Node.js** (18+) 
- **MySQL** (8.0+)

### 启动项目

```bash
# 克隆项目后
cd mcp-gateway

# 使用启动脚本（推荐）
./scripts/deployment/start.sh

# 或手动启动
cargo build --release
cd web && npm run build && cd ..
./target/release/mcp-gateway
```

## 📋 功能特性

### ✅ 已完成功能

1. **现代化前端界面**
   - 基于 React + TypeScript + Tailwind CSS
   - 响应式设计，支持深色模式
   - 使用 Shadcn UI 组件库
   - 毛玻璃效果和流畅动画

2. **端点管理**
   - 查看、添加、删除、更新 MCP 端点服务
   - 端点详细信息显示
   - 状态管理（运行中、已停用、已删除）

3. **指标监控**
   - 实时监控系统指标
   - 连接数、请求数、响应时间统计
   - 可视化数据展示

4. **连接计数**
   - 实时统计SSE和Streamable会话连接数
   - 自动更新数据库中的连接计数

5. **优雅停机**
   - 完整的连接跟踪机制
   - 配置化超时时间
   - 强制停机支持
   - 信号处理（SIGTERM, SIGINT）

6. **API 端点**
   - `/api/endpoint` - 端点管理
   - `/api/swagger` - Swagger 转换
   - `/api/health` - 健康检查
   - `/api/system/status` - 系统状态

### 🔄 技术架构

**后端技术栈：**
- Rust (Edition 2021)
- Axum Web 框架
- SQLx ORM + MySQL
- Tokio 异步运行时
- Serde 序列化
- Tracing 日志

**前端技术栈：**
- React 18 + TypeScript
- Vite 构建工具
- Tailwind CSS
- Shadcn UI 组件
- Axios HTTP 客户端

## 🛠 开发指南

### 项目结构

```
mcp-gateway/
├── src/                    # Rust 后端源码
│   ├── handlers/          # API 处理器
│   ├── services/          # 业务逻辑
│   ├── models/           # 数据模型
│   ├── middleware/       # 中间件
│   └── utils/           # 工具函数
├── web/                   # React 前端源码
│   ├── src/
│   │   ├── components/   # React 组件
│   │   ├── services/     # API 服务
│   │   └── types/       # TypeScript 类型
│   └── public/          # 静态资源
├── scripts/               # 项目脚本
│   ├── deployment/       # 部署脚本
│   ├── testing/          # 测试脚本
│   └── database/         # 数据库脚本
├── docs/                  # 项目文档
├── database/              # 数据库初始化脚本
├── migrations/           # 数据库迁移文件
├── docker/               # Docker 配置文件
├── spec/                  # 项目规格文档
└── monitoring/           # 监控配置
```

### 环境变量

```bash
# 数据库配置
DATABASE_URL=mysql://user:password@localhost:3306/mcp_gateway

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# 优雅停机配置
SHUTDOWN_TIMEOUT=30
FORCE_SHUTDOWN=false
```

## 🧪 测试

``bash
# 后端测试
cargo test

# 前端测试
cd web && npm test

# 构建测试
cargo build --release
cd web && npm run build
```

## 📊 监控端点

- **健康检查**: `GET /api/health`
- **系统状态**: `GET /api/system/status`
- **端点列表**: `GET /api/endpoint`

## 🔧 配置

项目支持通过 `config.toml` 文件或环境变量进行配置：

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "mysql://localhost:3306/mcp_gateway"
max_connections = 10

[monitoring]
enabled = true
metrics_path = "/metrics"
```

## 🚦 部署

### Docker 部署

项目提供了 Docker 配置文件，可以使用 Docker 部署服务：

```bash
# 构建镜像
docker build -t mcp-gateway -f docker/Dockerfile .

# 运行容器
docker run -p 3000:3000 mcp-gateway
```

### Docker Compose 部署

使用 Docker Compose 可以更方便地管理多个服务：

```bash
# 启动中间件服务
docker-compose -f docker/docker-compose.middleware.yml up -d

# 启动主应用服务
docker-compose -f docker/docker-compose.yml up -d
```

更多 Docker 使用说明请查看 [Docker 配置说明](./docker/README.md)

## 📈 性能特性

- **连接跟踪**: 实时监控活跃连接数
- **优雅停机**: 确保请求完整处理
- **内存安全**: Rust 保证的内存安全
- **异步处理**: 基于 Tokio 的高性能异步处理

## 🤝 贡献

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证。详情请参阅 [LICENSE](LICENSE) 文件。

## 🆘 常见问题

**Q: 如何配置数据库？**
A: 设置 `DATABASE_URL` 环境变量或在 `config.toml` 中配置数据库连接。

**Q: 前端页面无法加载？**
A: 确保已运行 `cd web && npm run build` 构建前端资产。

**Q: 如何启用监控？**
A: 在配置文件中设置 `monitoring.enabled = true`。

---

**项目状态**: ✅ 可用于开发和测试
**最后更新**: 2024年9月