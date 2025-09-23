# 数据库相关文件

此目录包含与数据库相关的初始化脚本和配置文件。

## 文件说明

- [init.sql](file:///Users/asura7969/dev/ai_project/mcp-gateway/database/init.sql) - 数据库初始化脚本，用于创建数据库和设置用户权限
- [migrations/](file:///Users/asura7969/dev/ai_project/mcp-gateway/migrations/) - 数据库迁移文件目录，包含表结构定义和更新脚本

## 使用说明

### 数据库初始化

运行 [init.sql](file:///Users/asura7969/dev/ai_project/mcp-gateway/database/init.sql) 脚本来创建数据库和用户：

```sql
-- 使用 root 用户连接到 MySQL
mysql -u root -p < database/init.sql
```

### 数据库迁移

数据库迁移文件按照数字顺序命名，表示应用的顺序：

1. [001_initial.sql](file:///Users/asura7969/dev/ai_project/mcp-gateway/migrations/001_initial.sql) - 初始表结构
2. [002_api_paths_unique.sql](file:///Users/asura7969/dev/ai_project/mcp-gateway/migrations/002_api_paths_unique.sql) - API 路径唯一性约束

在应用程序启动时会自动应用这些迁移。