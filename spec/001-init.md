# mcp网关
## 介绍
mcp网关是一个基于rust的网关，用于处理mcp协议的请求。

## 技术选型
### 后端
 - 语言：rust（Edition 2021）
 - web框架：axum（version 0.8.4）
 - ORM：sqlx（version 0.8）
 - 序列化：serde
 - 日志：tracing-subscriber（version 0.3）
 - 时间处理：chrono
 - 协议：mcp、http
 - 数据库：mysql
 - 监控：axum-prometheus（version 0.9.0）
 - 生成目录位置：../src

### 前端
 - 语言：typescript
 - 框架：react
 - 前端工具链：vite
 - 组件库：shadcn ui
 - 协议：http
 - 库：axios
 - 样式：tailwindcss
 - 生成目录位置：../web
 - 前端布局基于[shadcn-admin](https://github.com/satnaing/shadcn-admin)改写

## 功能

### 页面功能
* 支持查看已配置的mcp端点服务;
    * 服务名称、接口名称、创建时间、当前状态（运行中、已停用）、当前连接数
* 支持添加、删除、更新mcp端点服务;
* 支持查看mcp端点服务的详细信息;
    * 服务名称、接口名称、创建时间、当前状态（运行中、已停用）、请求参数（标注必填、非必填、参数描述、参数类型）、响应格式
* 支持查看mcp端点服务的配置信息;
* 支持删除mcp端点服务；
    * 删除端点服务时，需注意已连接的请求，需等待连接断开后删除；
    * 或需要删除的端点服务标记已删除，不支持新的连接，待老连接断开有正式删除;
* 支持更新mcp端点服务的配置信息;
* 支持查看mcp端点服务的指标信息;
    * 指标信息包括：请求数、响应数、响应时间、错误数、当前连接数、连接时长;
    * 指标信息可通过prometheus查询，grafana展示;

### 接口列表
- /api/swagger：Swagger接口文档转换为mcp端点服务接口，用于将Swagger接口文档转换为mcp端点服务;
- /api/endpoint：mcp端点服务接口，用于调用mcp端点服务;
- /api/endpoint/{id}：mcp端点服务接口，用于获取mcp端点服务的详细信息;
- /api/endpoint/{id}/config：mcp端点服务接口，用于获取mcp端点服务的配置信息;
- /api/endpoint/{id}/config：mcp端点服务接口，用于更新mcp端点服务的配置信息;
- /api/endpoint/{id}/config：mcp端点服务接口，用于删除mcp端点服务;

### Swagger json（或yaml）转换为mcp端点服务
- 用户输入的内容存储到数据库;
- 要求支持的mcp服务支持端点支持sse与streamable供客户端调用;
- 用户输入一段swagger3.x的yaml或json内容，网关根据内容自动生成支持mcp协议的服务端点;
- 页面上展示mcp-client的配置信息（mcp-remote-server配置信息）；

### 优雅停机 & 重启
参考：https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs

* 支持优雅停机;
    * 停机时，需等待已连接的请求处理完成后停机;
    * 或需要停机的端点服务标记已停机，不支持接受新的连接，待老连接断开后正式停机;
* 支持重启和强制重启;
    * 默认优雅重启；强制重启需添加参数`--force`;
    * 重启时，需等待已连接的请求处理完成后重启（非强制重启）;
    * 或需要重启的端点服务标记已重启，不支持新的连接，待老连接断开有正式重启（非强制重启）;

### 部署方式
本地docker部署

### 服务部署
- 项目支持docker-compose部署
    * mysql
    * prometheus
    * grafana
    * mcp网关（本项目）
    * 前端（本项目）

### 服务监控
- 项目支持prometheus、grafana监控

## 长期维护功能
* 编写接口单元测试