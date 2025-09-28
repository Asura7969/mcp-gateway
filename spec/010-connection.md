# 统计sse & streamable会话连接数

目标：实时统计各个endpoint的sse & streamable会话连接数，存储到mysql。

背景：本项目后端依赖库使用的`rmcp` crate，源代码在`/Users/asura7969/dev/rust_project/rust-sdk`, 每次请求后端接口`/{endpoint_id}/sse`(sse) & `/stream/{endpoint_id}`（streamable），依赖库内部会创建session_id，且当客户端断开连接时，会删除session_id。

实现方案：

如下streamable协议的log日志(log_requests方法):

新session请求
```text
2025-09-24T12:55:10.537739Z  INFO mcp_gateway::middleware::logging: Incoming request: POST /stream/0b88fc39-16c8-4238-bee8-11503522ba95
...
2025-09-24T12:55:10.537869Z  INFO mcp_gateway::middleware::logging: Header: mcp-session-id: 1337f2bb-c865-4b49-9f0d-a167647210cd
```


客户端主动断开关闭session请求
```text
2025-09-24T12:57:18.199907Z  INFO mcp_gateway::middleware::logging: Incoming request: DELETE /stream/0b88fc39-16c8-4238-bee8-11503522ba95
2025-09-24T12:57:18.199973Z  INFO mcp_gateway::middleware::logging: Header: mcp-session-id: 1337f2bb-c865-4b49-9f0d-a167647210cd
```


如下是sse协议的请求日志
```text
2025-09-24T13:29:06.639840Z  INFO mcp_gateway::middleware::logging: Incoming request: POST /message?sessionId=0b8af9cd-26ce-4f10-b8d4-674e7cf39e40&endpointId=0b88fc39-16c8-4238-bee8-11503522ba95
```
> 注：sse协议暂时只有创建连接的日志，端开连接逻辑先可以不实现



每次有新session连接时，我需要记录哪个endpoint_id，新连接对应的mcp-session-id，然后对应endpoint_id的连接数+1，同步到数据库；同理，每次客户端主动断开关闭session请求，需要删除数据库中对应的endpoint_id session_id，然后endpoint_id的连接数-1;

你需要创建2张表
```sql
-- endpoint_id, session_id映射表，endpoint_id & session_id为唯一索引
-- 新请求创建时，connect_at与disconnect_at时间一致，连接断开后，修改disconnect_at时间
CREATE TABLE IF NOT EXISTS endpoint_logs_v2 (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    session_id CHAR(36) NOT NULL,
    transport_type smallint(2) NOT NULL COMMENT '1:sse, 2:streamable',
    connect_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    disconnect_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_session_id (session_id),
    INDEX idx_connect_at (connect_at),
    INDEX idx_disconnect_at (disconnect_at)
);

-- endpoint连接数量表(此表主要针对实时查询，当前时间各个endpoint的连接数量)
CREATE TABLE IF NOT EXISTS endpoint_logs_v2 (
    id CHAR(36) PRIMARY KEY,
    endpoint_id CHAR(36) NOT NULL,
    connect_num bigint DEFAULT 0,
    INDEX idx_endpoint_id (endpoint_id),
    INDEX idx_session_id (session_id),
    INDEX idx_connect_at (connect_at),
    INDEX idx_disconnect_at (disconnect_at)
);
```

需对外提供查询接口:
- 查询指定时间段内，指定endpoint的连接信息
- 查询指定endpoint_id的总连接数
- 查询指定时间段内，各个endpoint每隔1分钟的连接数


> 数据库连接信息：mysql://mcpuser:mcppassword@localhost:3306/mcp_gateway

