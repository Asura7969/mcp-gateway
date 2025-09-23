#!/bin/bash

# MCP Gateway 测试脚本
# 用于运行单元测试和集成测试

set -e

echo "======================================"
echo "MCP Gateway 测试套件"
echo "======================================"

# 检查测试数据库环境变量
if [ -z "$TEST_DATABASE_URL" ]; then
    echo "警告: 未设置 TEST_DATABASE_URL 环境变量"
    echo "使用默认测试数据库: mysql://mcpuser:mcppassword@localhost:3306/mcp_gateway_test"
    export TEST_DATABASE_URL="mysql://mcpuser:mcppassword@localhost:3306/mcp_gateway_test"
fi

echo "测试数据库: $TEST_DATABASE_URL"
echo ""

# 编译项目
echo "1. 编译项目..."
cargo build --tests
echo "✓ 编译完成"
echo ""

# 运行单元测试（不需要数据库的测试）
echo "2. 运行单元测试..."
echo "运行 MCP 请求验证测试..."
cargo test test_mcp_request_validation --lib
echo "✓ MCP 请求验证测试通过"

echo "运行 MCP 错误响应测试..."
cargo test test_mcp_error_responses --lib
echo "✓ MCP 错误响应测试通过"

echo "运行传输协议枚举测试..."
cargo test test_transport_protocol_enum --lib
echo "✓ 传输协议枚举测试通过"
echo ""

# 运行基本功能测试
echo "3. 运行基本功能测试..."
echo "运行健康检查测试..."
cargo test --lib health
echo "✓ 健康检查测试通过"
echo ""

# 显示集成测试信息（需要数据库）
echo "4. 集成测试信息..."
echo "以下测试需要测试数据库，使用 --ignored 标志跳过："
echo ""
echo "   - MCP 多协议传输测试:"
echo "     * test_mcp_stdio_transport"
echo "     * test_mcp_sse_transport" 
echo "     * test_mcp_streamable_transport"
echo "     * test_mcp_websocket_transport"
echo ""
echo "   - 端点管理集成测试:"
echo "     * test_endpoint_crud_operations"
echo "     * test_endpoints_pagination"
echo "     * test_endpoint_status_filtering"
echo ""

# 如果有数据库访问权限，可以运行集成测试
echo "5. 尝试运行集成测试（如果数据库可用）..."

# 检查数据库连接
if command -v mysql >/dev/null 2>&1; then
    if mysql -h localhost -u mcpuser -pmcppassword -e "USE mcp_gateway_test;" 2>/dev/null; then
        echo "✓ 测试数据库连接成功，运行集成测试..."
        
        echo "运行端点无效操作测试..."
        cargo test test_invalid_endpoint_operations --test endpoint_integration_tests
        echo "✓ 端点无效操作测试通过"
        
        echo ""
        echo "注意: 数据库相关的测试已被标记为 #[ignore]，需要手动运行："
        echo "cargo test --test mcp_transport_tests -- --ignored"
        echo "cargo test --test endpoint_integration_tests -- --ignored"
    else
        echo "⚠ 无法连接到测试数据库，跳过数据库相关测试"
    fi
else
    echo "⚠ 未找到 mysql 客户端，跳过数据库连接检查"
fi

echo ""
echo "======================================"
echo "测试完成总结"
echo "======================================"
echo "✓ 单元测试: 通过"
echo "✓ 基本功能测试: 通过"
echo "ⓘ 集成测试: 需要测试数据库（已跳过）"
echo ""
echo "要运行完整的集成测试，请确保："
echo "1. MySQL 测试数据库正在运行"
echo "2. 设置 TEST_DATABASE_URL 环境变量"
echo "3. 运行: cargo test -- --ignored"
echo ""
echo "项目状态: ✅ 所有可用测试通过"