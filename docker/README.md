# Docker 配置文件

此目录包含项目的所有 Docker 相关配置文件。

## 文件说明

- [Dockerfile](file:///Users/asura7969/dev/ai_project/mcp-gateway/docker/Dockerfile) - 用于构建 MCP Gateway 应用镜像的 Dockerfile
- [docker-compose.yml](file:///Users/asura7969/dev/ai_project/mcp-gateway/docker/docker-compose.yml) - 主应用服务的 Docker Compose 配置
- [docker-compose.middleware.yml](file:///Users/asura7969/dev/ai_project/mcp-gateway/docker/docker-compose.middleware.yml) - 中间件服务（MySQL、Prometheus、Grafana）的 Docker Compose 配置

## 使用说明

### 构建 Docker 镜像

```bash
# 构建应用镜像
docker build -t mcp-gateway -f docker/Dockerfile .
```

### 使用 Docker Compose 启动服务

```bash
# 启动中间件服务
docker-compose -f docker/docker-compose.middleware.yml up -d

# 启动主应用服务
docker-compose -f docker/docker-compose.yml up -d
```

### 完整启动所有服务

项目提供了启动脚本，可以一键启动所有服务：

```bash
# 使用项目脚本启动所有服务
./scripts/deployment/start.sh
```