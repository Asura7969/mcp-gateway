#!/bin/bash

# MCP Gateway 停止脚本

set -e

echo "🛑 停止 MCP Gateway 服务..."

# 停止应用服务
echo "📱 停止应用服务..."
docker-compose down

# 停止中间件服务
echo "🗄️ 停止中间件服务..."
docker-compose -f docker-compose.middleware.yml down

echo "✅ 所有服务已停止"
echo ""
echo "💡 提示："
echo "  - 重新启动: ./start.sh"
echo "  - 完全清理 (包括数据): ./clean.sh"
echo "  - 查看剩余容器: docker ps -a"