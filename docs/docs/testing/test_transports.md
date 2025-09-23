# MCP Gateway 多传输协议测试指南

本文档介绍如何测试MCP Gateway支持的多种传输协议。

## 支持的传输协议

### 1. Stdio传输协议
- **端点**: `/mcp/{endpoint_id}/stdio`
- **用途**: 命令行客户端，标准输入输出
- **返回**: 可执行的Shell脚本

### 2. Server-Sent Events (SSE)
- **端点**: `/mcp/{endpoint_id}/sse`
- **用途**: 服务器向客户端的单向流式推送
- **格式**: text/event-stream

### 3. Streamable传输协议
- **端点**: `/mcp/{endpoint_id}/streamable`
- **方法**: POST
- **用途**: 流式响应处理，支持长时间运行的任务
- **格式**: application/x-ndjson

## 测试步骤

### 前置条件
1. 启动MCP Gateway服务器
2. 创建一个MCP端点并获取endpoint_id

### 测试Stdio传输
```bash
# 获取stdio脚本
curl -X GET http://localhost:3000/mcp/{endpoint_id}/stdio

# 保存并执行脚本
curl -X GET http://localhost:3000/mcp/{endpoint_id}/stdio > mcp_stdio.sh
chmod +x mcp_stdio.sh
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}' | ./mcp_stdio.sh
```

### 测试SSE传输
```bash
# 连接SSE流
curl -N -H "Accept: text/event-stream" http://localhost:3000/mcp/{endpoint_id}/sse
```

### 测试Streamable传输
```bash
# 发送工具调用请求，获取流式响应
curl -X POST http://localhost:3000/mcp/{endpoint_id}/streamable \
  -H "Content-Type: application/json" \
  -N \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "example_tool",
      "arguments": {"param1": "value1"}
    }
  }'
```

## 响应格式示例

### 初始化响应
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "mcp-endpoint-name",
      "version": "1.0.0"
    }
  }
}
```

### 工具列表响应
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "example_tool",
        "description": "An example tool",
        "inputSchema": {
          "type": "object",
          "properties": {
            "param1": {"type": "string"}
          }
        }
      }
    ]
  }
}
```

### 流式响应（Streamable）
```json
{"jsonrpc": "2.0", "id": 1, "result": {"type": "progress", "message": "Executing tool: example_tool"}}
{"jsonrpc": "2.0", "id": 1, "result": {"content": [{"type": "text", "text": "Tool execution completed"}]}}
```

## 错误处理

各传输协议都支持标准的MCP错误响应格式：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

## 注意事项

1. 所有传输协议都要求端点处于运行状态
2. SSE 支持长连接
3. Stdio 适合命令行集成
4. Streamable 适合处理长时间运行的任务
5. 请确保在测试前先创建并启动MCP端点