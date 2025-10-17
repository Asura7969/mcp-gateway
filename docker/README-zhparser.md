# pgvecto-rs with zhparser 中文分词插件

本配置在原有的 pgvecto-rs 基础上添加了 zhparser 中文分词插件，支持中文全文搜索功能。

## 文件说明

- `Dockerfile.pgvecto-rs-zhparser`: 自定义 Dockerfile，基于 pgvecto-rs 镜像安装 zhparser 插件
- `init-zhparser.sql`: 数据库初始化脚本，自动创建中文分词配置和测试表
- `docker-compose.middleware.yml`: 更新后的 Docker Compose 配置

## 构建和启动

1. 构建自定义镜像并启动服务：
```bash
cd docker
docker-compose -f docker-compose.middleware.yml --profile pgvecto-rs up --build -d
```

2. 检查服务状态：
```bash
docker-compose -f docker-compose.middleware.yml ps
```

## 验证中文分词功能

连接到数据库后，可以使用以下 SQL 命令测试中文分词：

```sql
-- 测试中文分词
SELECT to_tsvector('chinese_zh', '中文分词测试');

-- 查看分词结果
SELECT to_tsvector('chinese_zh', '这是一个中文分词的测试文档');

-- 全文搜索测试
SELECT * FROM test_chinese_search 
WHERE search_vector @@ to_tsquery('chinese_zh', '中文 & 分词');

-- 查看可用的文本搜索配置
SELECT cfgname FROM pg_ts_config;
```

## 主要特性

1. **中文分词支持**: 使用 SCWS (Simple Chinese Word Segmentation) 进行中文分词
2. **全文搜索**: 支持中文全文搜索和相关性排序
3. **向量搜索**: 保留原有的 pgvecto-rs 向量搜索功能
4. **自动初始化**: 容器启动时自动配置中文分词环境

## 环境变量

- `SCWS_HOME`: SCWS 安装路径，默认为 `/usr/local/scws`
- 其他环境变量与原 pgvecto-rs 配置相同

## 注意事项

1. 首次构建可能需要较长时间，因为需要编译 SCWS 和 zhparser
2. 确保有足够的磁盘空间用于构建过程
3. 中文词典文件会在构建时自动下载

## 故障排除

如果遇到问题，可以查看容器日志：
```bash
docker-compose -f docker-compose.middleware.yml logs pgvecto-rs
```

检查扩展是否正确安装：
```sql
SELECT * FROM pg_extension WHERE extname = 'zhparser';
```