-- Add index_mapping column to store ES index mapping JSON per dataset
ALTER TABLE `t_dataset`
    ADD COLUMN `index_mapping` TEXT NULL AFTER `table_schema`;