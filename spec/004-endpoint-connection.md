# 任务列表

## 端点管理页面，点击某个服务名称，跳转新页面到该服务mcp-server的调试页面
### 调试页面功能
* 支持sse、stdio、streamable调试该项目提供的mcp-server;
* 支持tools、resources调试;
  * 查看tools工具列表
  * 调用工具并展示响应结果
  * 查看resources资源列表
* 停用状态的服务不支持跳转到调试页面，并且友好提示用户;
> 页面功能及布局参考官方：https://modelcontextprotocol.io/legacy/tools/inspector

## 改进项
### 页面（dailog）宽度扩大0.8倍
* MCP客户端配置页面
* 编辑端点
* 创建端点

### MCP 客户端配置页，配置内容sse协议是否需要`transport`包裹
现状：
```json
{
  "mcpServers": {
    "agent-bot": {
      "transport": {
        "type": "sse",
        "url": "http://localhost:3000/0b88fc39-16c8-4238-bee8-11503522ba95/sse"
      }
    }
  }
}
```
是否需要改成：
```json
{
  "mcpServers": {
    "agent-bot": {
      "type": "sse",
      "url": "http://localhost:3000/0b88fc39-16c8-4238-bee8-11503522ba95/sse"
    }
  }
}
```

streamable协议同理。
默认有限展示sse协议配置。