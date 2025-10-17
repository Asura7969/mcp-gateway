-- 初始化 zhparser 扩展
-- 这个脚本会在数据库初始化时自动执行

-- 创建 zhparser 扩展
CREATE EXTENSION IF NOT EXISTS zhparser;

-- 创建中文分词配置
CREATE TEXT SEARCH CONFIGURATION chinese_zh (PARSER = zhparser);

-- 添加中文分词的 token 映射
ALTER TEXT SEARCH CONFIGURATION chinese_zh ADD MAPPING FOR n,v,a,i,e,l WITH simple;

-- 设置默认的文本搜索配置为中文
-- ALTER DATABASE mcp SET default_text_search_config = 'chinese_zh';

-- 创建一个测试表来验证中文分词功能
CREATE TABLE IF NOT EXISTS test_chinese_search (
    id SERIAL PRIMARY KEY,
    title TEXT,
    content TEXT,
    search_vector tsvector
);

-- 创建触发器函数来自动更新搜索向量
CREATE OR REPLACE FUNCTION update_search_vector() RETURNS trigger AS $$
BEGIN
    NEW.search_vector := to_tsvector('chinese_zh', COALESCE(NEW.title, '') || ' ' || COALESCE(NEW.content, ''));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
DROP TRIGGER IF EXISTS update_search_vector_trigger ON test_chinese_search;
CREATE TRIGGER update_search_vector_trigger
    BEFORE INSERT OR UPDATE ON test_chinese_search
    FOR EACH ROW EXECUTE FUNCTION update_search_vector();

-- 创建搜索索引
CREATE INDEX IF NOT EXISTS idx_search_vector ON test_chinese_search USING gin(search_vector);

-- 插入测试数据
INSERT INTO test_chinese_search (title, content) VALUES 
('中文分词测试', '这是一个中文分词的测试文档，用于验证zhparser插件是否正常工作。'),
('全文搜索功能', 'PostgreSQL的全文搜索功能结合中文分词可以提供更好的搜索体验。'),
('向量数据库', 'pgvecto-rs是一个高性能的向量数据库扩展，支持相似性搜索。')
ON CONFLICT DO NOTHING;

-- 显示配置信息
\echo '中文分词配置已完成！'
\echo '可以使用以下命令测试中文分词：'
\echo 'SELECT to_tsvector(''chinese_zh'', ''中文分词测试'');'