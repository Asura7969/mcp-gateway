#!/bin/bash

# MCP Gateway 项目启动脚本

set -e

echo "🚀 启动 MCP Gateway 项目..."

# 检查是否安装了必要的工具
check_dependencies() {
    echo "📋 检查依赖..."
    
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo 未安装"
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        echo "❌ Node.js/npm 未安装"
        exit 1
    fi
    
    echo "✅ 依赖检查完成"
}

# 构建后端
build_backend() {
    echo "🔨 构建后端..."
    cargo build --release
    echo "✅ 后端构建完成"
}

# 构建前端
build_frontend() {
    echo "🔨 构建前端..."
    cd web
    npm run build
    cd ..
    echo "✅ 前端构建完成"
}

# 检查配置
check_config() {
    echo "⚙️  检查配置..."
    
    # 检查是否有配置文件
    if [ ! -f "config.toml" ]; then
        echo "⚠️  未找到 config.toml，使用默认配置"
    fi
    
    echo "✅ 配置检查完成"
}

# 启动服务
start_server() {
    echo "🌐 启动 MCP Gateway 服务器..."
    echo "📍 服务将在 http://localhost:3000 启动"
    echo "📍 API 文档: http://localhost:3000/api/health"
    echo "📍 系统状态: http://localhost:3000/api/system/status"
    echo ""
    echo "按 Ctrl+C 停止服务器"
    echo ""
    
    ./target/release/mcp-gateway
}

# 主函数
main() {
    check_dependencies
    build_backend
    build_frontend
    check_config
    start_server
}

# 执行主函数
main "$@"