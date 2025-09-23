# MCP 客户端配置动态域名使用指南

本指南说明如何在 MCP Gateway 前端配置动态后端域名，确保 MCP 客户端配置显示正确的端点地址。

## 问题描述

之前的版本中，MCP 客户端配置页面显示的端点地址被硬编码为 `http://localhost:3000`，这在部署到不同环境时会导致配置不正确。

## 解决方案

### 1. 环境变量配置

在前端项目的环境配置文件中设置 `VITE_API_BASE_URL`：

#### 开发环境 (`.env.development`)
```bash
VITE_API_BASE_URL=http://localhost:3000
```

#### 测试环境 (`.env.test`)
```bash
VITE_API_BASE_URL=http://test-api.example.com:8080
```

#### 生产环境 (`.env.production`)
```bash
VITE_API_BASE_URL=https://api.production.com
```

#### 自定义环境
```bash
# 例如：内网部署
VITE_API_BASE_URL=http://192.168.1.100:3000

# 例如：云服务器部署
VITE_API_BASE_URL=https://mcp-gateway.yourdomain.com
```

### 2. 代码实现

前端代码现在使用动态环境变量：

```typescript
// 之前的硬编码方式
const baseUrl = 'http://localhost:3000'

// 现在的动态方式
const baseUrl = env.apiBaseUrl
```

### 3. 生成的配置示例

根据不同的 `VITE_API_BASE_URL` 设置，生成的 MCP 客户端配置如下：

#### stdio 协议
```json
{
  "mcpServers": {
    "your-endpoint-name": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "-H", "Content-Type: application/json",
        "-H", "Accept: application/json",
        "--data-binary", "@-",
        "https://api.production.com/mcp/{endpoint_id}/stdio"
      ]
    }
  }
}
```

#### SSE 协议
```json
{
  "mcpServers": {
    "your-endpoint-name": {
      "transport": {
        "type": "sse",
        "url": "https://api.production.com/mcp/{endpoint_id}/sse"
      }
    }
  }
}
```

#### HTTP Stream 协议
```json
{
  "mcpServers": {
    "your-endpoint-name": {
      "transport": {
        "type": "http",
        "url": "https://api.production.com/mcp/{endpoint_id}/stream",
        "method": "POST",
        "headers": {
          "Content-Type": "application/json",
          "Accept": "text/event-stream"
        }
      }
    }
  }
}
```

## 使用步骤

### 1. 设置环境变量

根据您的部署环境，创建或修改相应的环境配置文件：

```bash
# 复制示例文件
cp .env.example .env.local

# 编辑配置文件
nano .env.local
```

### 2. 修改后端 API 地址

```bash
# 修改为您的实际后端地址
VITE_API_BASE_URL=https://your-backend-domain.com:port
```

### 3. 重启前端服务

```bash
# 重启开发服务器
npm run dev

# 或重新构建生产版本
npm run build
```

### 4. 验证配置

1. 打开端点管理页面
2. 点击任意端点的"配置"按钮
3. 验证显示的端点地址是否使用了正确的域名和端口

## 注意事项

1. **协议匹配**：确保 `VITE_API_BASE_URL` 中的协议（http/https）与后端服务匹配
2. **端口配置**：如果后端使用非标准端口，请在 URL 中明确指定
3. **域名解析**：确保客户端能够解析和访问配置的域名
4. **CORS 设置**：确保后端 CORS 配置允许来自前端域名的请求
5. **SSL 证书**：如果使用 HTTPS，确保 SSL 证书有效

## 常见问题

### Q: 修改环境变量后没有生效？
A: 需要重启前端开发服务器，环境变量在构建时注入。

### Q: 生产环境如何设置？
A: 在构建时设置环境变量：
```bash
VITE_API_BASE_URL=https://api.production.com npm run build
```

### Q: 能否在运行时动态修改？
A: 不能，Vite 的环境变量在构建时注入，运行时不可修改。

## 测试验证

您可以通过以下方式验证配置是否正确：

1. **前端控制台**：查看网络请求是否指向正确的 API 地址
2. **配置页面**：检查生成的 MCP 客户端配置中的 URL
3. **端点测试**：使用生成的配置测试 MCP 客户端连接

## 相关文件

- 环境配置：`web/.env.*`
- 环境模块：`web/src/lib/env.ts`
- 配置组件：`web/src/routes/_authenticated/endpoints.tsx`
- 配置文档：`web/ENV_CONFIG.md`