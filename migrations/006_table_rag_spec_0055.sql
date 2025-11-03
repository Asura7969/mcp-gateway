-- Spec 0055: add retrieval/reply columns, adjust defaults, and add index_name
ALTER TABLE `t_dataset`
    ADD COLUMN `retrieval_column` TEXT DEFAULT NULL COMMENT '参与检索列,多个列用逗号分隔';

ALTER TABLE `t_dataset`
    ADD COLUMN `reply_column` TEXT DEFAULT NULL COMMENT '参与回复列,多个列用逗号分隔';

-- Adjust defaults to align with spec (3 results, threshold 0.1)
ALTER TABLE `t_dataset`
    MODIFY COLUMN `max_results` INT(5) DEFAULT 3 COMMENT '召回数量';

ALTER TABLE `t_dataset`
    MODIFY COLUMN `similarity_threshold` DOUBLE DEFAULT 0.1 COMMENT '相似度阈值';

-- Add index_name to store ES index name (distinct from user-provided table_name)
ALTER TABLE `t_dataset`
    ADD COLUMN `index_name` VARCHAR(100) DEFAULT '' COMMENT 'ES索引名称';

-- Backfill index_name from existing table_name values for compatibility
UPDATE `t_dataset`
SET `index_name` = `table_name`
WHERE (`index_name` IS NULL OR `index_name` = '');