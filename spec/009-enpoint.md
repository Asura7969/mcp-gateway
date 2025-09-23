# 优化端点管理页面

列表页面展示字段优化：
* ID、状态列去除
* 添例如Tash页的筛选功能
* 描述列需展示服务的方法个数
* 添加当前服务的连接数
* 点击查看、编辑、删除按钮并无响应

端点管理页面需兼容后端接口返回的响应格式，并展示数据
```rust
        .route("/api/endpoint", post(create_endpoint).get(list_endpoints))
        .route("/api/endpoints", get(list_endpoints_paginated))
        .route(
            "/api/endpoint/{id}",
            get(get_endpoint)
                .put(update_endpoint)
                .delete(delete_endpoint),
        )
        .route("/api/endpoint/{id}/metrics", get(get_endpoint_metrics))
        .route("/api/endpoint/{id}/start", post(start_endpoint))
        .route("/api/endpoint/{id}/stop", post(stop_endpoint))
```
后端项目在`src`目录下

严格按照./003-web-list-endpoint.md中的描述还原页面
