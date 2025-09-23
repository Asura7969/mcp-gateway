# MCP Gateway 多传输协议快速入门

## 功能总览 ✅

MCP Gateway现在完全支持四种传输协议：

- ✅ **WebSocket** - 双向实时通信
- ✅ **Stdio** - 命令行集成
- ✅ **Server-Sent Events (SSE)** - 单向流式推送
- ✅ **Streamable** - 流式响应处理

## 快速开始

### 1. 启动服务器
```bash
cd /Users/asura7969/dev/ai_project/mcp-gateway
cargo run
```

### 2. 创建MCP端点
```bash
# 创建一个示例端点
curl -X POST http://localhost:3000/api/endpoint \
  -H "Content-Type: application/json" \
  -d '{
    "name": "example-api",
    "description": "示例API端点",
    "swagger_content": "{\"openapi\": \"3.0.0\", \"info\": {\"title\": \"Example API\", \"version\": \"1.0.0\"}, \"paths\": {\"/test\": {\"get\": {\"summary\": \"测试端点\"}}}}"
  }'
```

### 3. 启动端点
```bash
# 获取创建的端点ID，然后启动
curl -X POST http://localhost:3000/api/endpoint/{endpoint_id}/start
```

### 4. 测试传输协议

#### 获取可用的传输端点
```bash
# 替换 {endpoint_id} 为实际的端点ID
ENDPOINT_ID="your-endpoint-id"

echo "可用的传输端点:"
echo "WebSocket:   ws://localhost:3000/mcp/$ENDPOINT_ID/ws"
echo "Stdio:       GET http://localhost:3000/mcp/$ENDPOINT_ID/stdio"
echo "SSE:         GET http://localhost:3000/mcp/$ENDPOINT_ID/sse"
echo "Streamable:  POST http://localhost:3000/mcp/$ENDPOINT_ID/streamable"
```

#### 使用测试脚本
```bash
# 使用我们提供的测试脚本
./test_mcp_transports.sh {endpoint_id}
```

## 传输协议详解

### WebSocket传输
```bash
# 需要安装wscat: npm install -g wscat
wscat -c ws://localhost:3000/mcp/{endpoint_id}/ws

# 发送初始化请求
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0.0"}}}
```

### Stdio传输
```bash
# 获取stdio脚本并执行
curl http://localhost:3000/mcp/{endpoint_id}/stdio > mcp_stdio.sh
chmod +x mcp_stdio.sh

# 通过管道发送MCP请求
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}' | ./mcp_stdio.sh
```

### SSE传输
```bash
# 连接SSE流
curl -N -H "Accept: text/event-stream" http://localhost:3000/mcp/{endpoint_id}/sse
```

### Streamable传输
```bash
# 发送流式请求
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

## 核心改进

### ✅ UUID存储格式修复
- 修复了MySQL CHAR(36)与Rust UUID的兼容性问题
- 实现了自定义FromRow trait处理字符串格式UUID

### ✅ 多传输协议支持
- 实现了完整的多传输协议架构
- 每种协议都有专门的处理器和路由

### ✅ 流式响应处理
- 支持长时间运行任务的进度反馈
- NDJSON格式的流式输出
- 异步非阻塞处理

### ✅ 错误处理改进
- 统一的MCP错误响应格式
- 传输协议特定的错误处理
- 详细的错误信息和调试日志

## 文件结构
```
src/
├── handlers/
│   ├── mcp_transport_handler.rs  # 多传输协议处理器 ✅
│   └── ...
├── services/
│   ├── mcp_service.rs           # MCP核心服务 ✅
│   └── ...
└── models/
    ├── endpoint.rs              # 端点模型(UUID修复) ✅
    └── ...

# 测试和文档
test_mcp_transports.sh           # 传输协议测试脚本 ✅
test_transports.md               # 测试指南 ✅
TRANSPORT_PROTOCOLS.md           # 架构文档 ✅
```

## 后续步骤

1. **测试验证**: 使用提供的测试脚本验证所有传输协议
2. **客户端集成**: 根据需要选择合适的传输协议
3. **监控配置**: 设置监控和日志记录
4. **生产部署**: 配置生产环境的数据库和网络

## 故障排除

### 常见问题
1. **端点未运行**: 确保端点状态为"running"
2. **连接失败**: 检查服务器是否正常启动
3. **权限问题**: 确保端点ID正确且有访问权限

### 日志查看
```bash
# 查看服务器日志
RUST_LOG=debug cargo run

# 查看特定模块日志
RUST_LOG=mcp_gateway::handlers::mcp_transport_handler=debug cargo run
```

这个实现完全满足了您的需求：对外提供的MCP Server支持stdio、SSE和streamable传输协议，并改进了之前的微调项！