#!/bin/bash

# MCP Gateway 多传输协议测试脚本
# 使用方法: ./test_mcp_transports.sh <endpoint_id>

set -e

ENDPOINT_ID=${1:-""}
BASE_URL="http://localhost:3000"

if [ -z "$ENDPOINT_ID" ]; then
    echo "错误: 请提供端点ID"
    echo "使用方法: $0 <endpoint_id>"
    exit 1
fi

echo "测试MCP Gateway多传输协议支持"
echo "端点ID: $ENDPOINT_ID"
echo "服务器: $BASE_URL"
echo "================================"

# 测试端点是否存在
echo "1. 检查端点状态..."
if curl -s -f "$BASE_URL/api/endpoint/$ENDPOINT_ID" > /dev/null; then
    echo "✓ 端点存在"
else
    echo "✗ 端点不存在或无法访问"
    exit 1
fi

# 测试Stdio传输
echo ""
echo "2. 测试Stdio传输协议..."
if curl -s -f "$BASE_URL/mcp/$ENDPOINT_ID/stdio" > /tmp/mcp_stdio.sh; then
    echo "✓ Stdio脚本生成成功"
    echo "  脚本保存到: /tmp/mcp_stdio.sh"
    chmod +x /tmp/mcp_stdio.sh
else
    echo "✗ Stdio传输测试失败"
fi

# 测试SSE传输
echo ""
echo "3. 测试SSE传输协议..."
timeout 5s curl -N -H "Accept: text/event-stream" "$BASE_URL/mcp/$ENDPOINT_ID/sse" &
SSE_PID=$!
sleep 2
if kill -0 $SSE_PID 2>/dev/null; then
    echo "✓ SSE连接建立成功"
    kill $SSE_PID 2>/dev/null || true
else
    echo "✗ SSE传输测试失败"
fi

# 测试Streamable传输
echo ""
echo "4. 测试Streamable传输协议..."
STREAMABLE_RESPONSE=$(curl -s -X POST "$BASE_URL/mcp/$ENDPOINT_ID/streamable" \
    -H "Content-Type: application/json" \
    -d '{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    }' 2>/dev/null || echo "")

if [ -n "$STREAMABLE_RESPONSE" ]; then
    echo "✓ Streamable传输测试成功"
    echo "  响应: $STREAMABLE_RESPONSE"
else
    echo "✗ Streamable传输测试失败"
fi

echo ""
echo "测试完成!"
echo ""
echo "可用的传输端点:"
echo "- Stdio:     GET $BASE_URL/mcp/$ENDPOINT_ID/stdio"
echo "- SSE:       GET $BASE_URL/mcp/$ENDPOINT_ID/sse"
echo "- Streamable: POST $BASE_URL/mcp/$ENDPOINT_ID/streamable"