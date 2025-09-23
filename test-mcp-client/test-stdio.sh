#!/bin/bash

# MCP Gateway stdio 协议测试脚本
# 测试通过 stdio 协议连接到 MCP Gateway 并调用 agent-bot 服务

GATEWAY_BASE_URL="http://localhost:3000"
ENDPOINT_ID="b0778a81-fba1-4d7b-9539-6d065eae6e22"
AGENT_ID="98e2b1cf-3a7d-4f6c-9b0a-5d8c7e6f5432"

echo "🧪 MCP Gateway Stdio 协议测试"
echo "═══════════════════════════════════════════════════════════"
echo "📡 端点地址: ${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/stdio"
echo "🎯 测试接口: /bot-agent/findByAgentId"
echo "📝 AgentId: ${AGENT_ID}"
echo "─────────────────────────────────────────────────────────"

echo ""
echo "📋 步骤 1: 获取可用工具列表"
echo "发送 MCP tools/list 请求..."

tools_request='{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}'

echo "请求内容: $tools_request"
echo ""

# 获取工具列表
echo "$tools_request" | curl -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  --data-binary "@-" \
  "${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/stdio" > tools_response.json

echo "✅ 工具列表响应已保存到 tools_response.json"
echo "📊 响应内容:"
cat tools_response.json | jq '.'

echo ""
echo "─────────────────────────────────────────────────────────"
echo "🔧 步骤 2: 调用 findByAgentId 工具"

# 从响应中提取工具名称
tool_name=$(cat tools_response.json | jq -r '.result.tools[] | select(.name | contains("findByAgentId") or contains("bot-agent") or contains("get_bot_agent_findbyagentid")) | .name' | head -1)

if [ -z "$tool_name" ] || [ "$tool_name" = "null" ]; then
    echo "❌ 未找到 findByAgentId 相关的工具"
    echo "可用工具列表:"
    cat tools_response.json | jq -r '.result.tools[].name'
    exit 1
fi

echo "🎯 找到目标工具: $tool_name"

call_request=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "$tool_name",
    "arguments": {
      "agentId": "$AGENT_ID"
    }
  }
}
EOF
)

echo "请求内容: $call_request"
echo ""

# 调用工具
echo "$call_request" | curl -X POST \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  --data-binary "@-" \
  "${GATEWAY_BASE_URL}/mcp/${ENDPOINT_ID}/stdio" > call_response.json

echo "✅ 工具调用响应已保存到 call_response.json"
echo "📊 响应内容:"
cat call_response.json | jq '.'

echo ""
echo "─────────────────────────────────────────────────────────"
echo "📋 步骤 3: 解析返回结果"

# 检查是否有错误
error=$(cat call_response.json | jq -r '.error')
if [ "$error" != "null" ]; then
    echo "❌ 工具调用失败:"
    cat call_response.json | jq '.error'
    exit 1
fi

# 解析返回的内容
echo "✅ 工具调用成功!"
echo ""
echo "📊 返回的内容:"
cat call_response.json | jq '.result.content[]'

echo ""
echo "🔍 尝试解析返回的文本内容为 JSON:"
text_content=$(cat call_response.json | jq -r '.result.content[] | select(.type=="text") | .text')
if [ -n "$text_content" ] && [ "$text_content" != "null" ]; then
    echo "$text_content" | jq '.' 2>/dev/null || echo "非 JSON 格式: $text_content"
else
    echo "未找到文本内容"
fi

echo ""
echo "🏁 测试完成!"
echo "📁 生成的文件:"
echo "  - tools_response.json: 工具列表响应"
echo "  - call_response.json: 工具调用响应"