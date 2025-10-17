# 支持`pgvecto-rs`向量检索

配置文件（`config/default.toml`）中通过vector_type选择哪种向量库，默认使用**pgvectorrs**

向量检索实现参考`/Users/asura7969/dev/python_project/dify/api/core/rag/datasource/vdb/pgvecto_rs`目录下的python实现

构造一个**Search trait**,用于向量检索、关键词检索和混合检索，
* 现有的`interface_retrieval_service`改为`surrealdb_service`,
* `InterfaceRetrievalService` 重命名为`SurrealdbService`
* `SurrealdbService`需要实现**Search trait**
* 新的`PgvectorRsService`需要实现**Search trait**
* 对`PgvectorRsService`的功能做单元测试, embedding使用阿里云百炼配置





