
# MCP
- [x] 支持swagger json转mcp
- [x] 支持sse、streamable mcp
- [x] 支持会话连接数统计
- [x] 支持接口向量检索
- [x] 支持阿里云百炼向量检测
- [x] ~~支持`SurrealDB`新增`rocksdb`存储~~
  - [x] ~~新增环境变量配置~~
- [x] ~~`SurrealDB`元数据过滤~~
  - [x] ~~server名称~~
  - [x] ~~method~~
  - [x] ~~path前缀路径~~
  - [x] ~~tag~~
- [ ] 支持**pgvector**向量&全文检索查询，通过配置文件动态选择向量数据库
  - [ ] 测试
  - [ ] 混合检索权重处理
- [x] 支持**elasticsearch**向量&全文检索查询，通过配置文件动态选择向量数据库
  - [x] 测试
  - [x] 混合检索权重处理
- [x] `RAG`检索后依据**chunk**查询接口详情
- [x] 前端新增向量检索接口调试页面
- [ ] 前端**search debug**页面重新对接后端接口返回格式
- [ ] 保存端点/修改端点,自动更新向量数据
- [ ] 测试会话连接数量准确性
- [x] mcp tools列表展示中文名称
- [ ] 支持prometheus指标输出
- [ ] grafana监控面板json配置文件
  - [ ] 服务状态
  - [ ] mcp会话连接数据
  - [ ] cpu、memory监控
- [ ] docker部署该项目
- [ ] readme重写


# RAG 数据表
- [ ] 需求文档 + 技术方案
- [ ] DuckDB集成
  - [ ] 上传文件接口, 存储文件元数据, 通过DuckDB写到阿里云OSS
    - [ ] 定义表`Schema`, 存储表元数据
    - [ ] 存储文件
    - [ ] 读取并存储文件数据
  - [ ] 读表数据, DuckDB读OSS数据