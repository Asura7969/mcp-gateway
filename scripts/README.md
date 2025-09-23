# MCP Gateway 脚本中心

欢迎来到 MCP Gateway 项目脚本中心。本文档将帮助您了解和使用项目中的各种脚本。

## 脚本分类

### 🚀 部署脚本 (deployment)

- [start.sh](./scripts/deployment/start.sh) - 项目启动脚本，包含依赖检查、构建和启动服务
- [stop.sh](./scripts/deployment/stop.sh) - 项目停止脚本，用于停止所有 Docker 服务
- [clean.sh](./scripts/deployment/clean.sh) - 项目清理脚本，用于完全清理所有数据和资源

### 🧪 测试脚本 (testing)

- [run_tests.sh](./scripts/testing/run_tests.sh) - 项目测试脚本，运行单元测试和集成测试
- [test_mcp_transports.sh](./scripts/testing/test_mcp_transports.sh) - MCP 多传输协议测试脚本

## 脚本使用说明

### 部署脚本

#### 启动项目
```bash
# 启动所有服务
./scripts/deployment/start.sh
```

#### 停止项目
```bash
# 停止所有服务
./scripts/deployment/stop.sh
```

#### 清理项目
```bash
# 完全清理所有数据和资源
./scripts/deployment/clean.sh
```

### 测试脚本

#### 运行测试
```bash
# 运行所有可用测试
./scripts/testing/run_tests.sh
```

#### 测试 MCP 传输协议
```bash
# 测试指定端点的 MCP 传输协议
./scripts/testing/test_mcp_transports.sh <endpoint_id>
```

## 注意事项

1. 所有脚本都需要在项目根目录下执行
2. 确保已安装必要的依赖（Rust, Node.js, Docker 等）
3. 测试脚本可能需要配置测试数据库环境变量
4. 清理脚本会删除所有数据，请谨慎使用

---
**提示**: 您可以直接从项目根目录通过 `./scripts/deployment/start.sh` 这样的路径来执行脚本。