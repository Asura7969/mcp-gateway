# MCP协议


协议版本2025-06-18

## mcp-server
后端实现参考根目录下`python-sdk`文件夹下`src/mcp/server/sse.py`,`src/mcp/server/streamable_http.py`,`src/mcp/server/stdio.py`的实现


## mcp-client

前端`Inspector`页面向后端发送请求遵循如下文件的协议：https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2025-06-18/schema.ts

代码可参考官方`inspector`的实现