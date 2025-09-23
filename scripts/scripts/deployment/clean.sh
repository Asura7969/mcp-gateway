#!/bin/bash

# MCP Gateway 清理脚本

set -e

echo "🧹 清理 MCP Gateway 所有数据..."
echo "⚠️  警告：这将删除所有数据，包括数据库数据！"
read -p "确认继续？(y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 操作已取消"
    exit 1
fi

# 停止所有服务
echo "🛑 停止所有服务..."
docker-compose down -v 2>/dev/null || true
docker-compose -f docker-compose.middleware.yml down -v 2>/dev/null || true

# 删除容器
echo "🗑️ 删除容器..."
docker rm -f mcp-gateway-backend mcp-gateway-frontend mcp-gateway-mysql mcp-gateway-prometheus mcp-gateway-grafana 2>/dev/null || true

# 删除数据卷
echo "💾 删除数据卷..."
docker volume rm mcp-gateway_mysql_data mcp-gateway_prometheus_data mcp-gateway_grafana_data 2>/dev/null || true

# 删除镜像（可选）
read -p "是否删除构建的镜像？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🖼️ 删除镜像..."
    docker rmi mcp-gateway_mcp-gateway mcp-gateway_mcp-frontend 2>/dev/null || true
fi

# 删除网络
echo "📡 删除网络..."
docker network rm mcp-network 2>/dev/null || true

echo "✅ 清理完成！"
echo ""
echo "💡 提示："
echo "  - 重新开始: ./start.sh"
echo "  - 查看剩余资源: docker system df"