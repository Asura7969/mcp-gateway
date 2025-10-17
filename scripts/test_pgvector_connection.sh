#!/bin/bash

# 测试 pgvecto-rs 数据库连接脚本

echo "🔍 测试 pgvecto-rs 数据库连接..."

# 数据库连接参数
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="postgres"
DB_PASSWORD="mcp123456"
DB_NAME="mcp"

# 检查 Docker 容器是否运行
echo "📋 检查 Docker 容器状态..."
if docker ps | grep -q "mcp-gateway-pgvecto-rs"; then
    echo "✅ pgvecto-rs 容器正在运行"
else
    echo "❌ pgvecto-rs 容器未运行"
    echo "请先启动容器: docker-compose -f docker/docker-compose.middleware.yml --profile pgvecto-rs up -d"
    exit 1
fi

# 检查端口是否开放
echo "🔌 检查端口 $DB_PORT 是否开放..."
if nc -z $DB_HOST $DB_PORT; then
    echo "✅ 端口 $DB_PORT 可访问"
else
    echo "❌ 端口 $DB_PORT 不可访问"
    exit 1
fi

# 测试数据库连接
echo "🔗 测试数据库连接..."
export PGPASSWORD=$DB_PASSWORD

if psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT version();" > /dev/null 2>&1; then
    echo "✅ 数据库连接成功"
    
    # 检查 pgvector 扩展
    echo "🧩 检查 pgvector 扩展..."
    if psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT * FROM pg_extension WHERE extname = 'vectors';" | grep -q "vectors"; then
        echo "✅ pgvector 扩展已安装"
    else
        echo "⚠️  pgvector 扩展未安装，尝试安装..."
        psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "CREATE EXTENSION IF NOT EXISTS vectors;"
        if [ $? -eq 0 ]; then
            echo "✅ pgvector 扩展安装成功"
        else
            echo "❌ pgvector 扩展安装失败"
        fi
    fi
    
    # 显示数据库信息
    echo "📊 数据库信息:"
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "
        SELECT 
            current_database() as database_name,
            current_user as current_user,
            version() as version;
    "
    
else
    echo "❌ 数据库连接失败"
    echo "请检查以下配置:"
    echo "  - 主机: $DB_HOST"
    echo "  - 端口: $DB_PORT"
    echo "  - 用户: $DB_USER"
    echo "  - 密码: $DB_PASSWORD"
    echo "  - 数据库: $DB_NAME"
    exit 1
fi

echo "🎉 所有检查完成！"