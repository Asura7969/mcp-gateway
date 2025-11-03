use crate::config::EmbeddingConfig;
use crate::models::{
    table_rag::{
        ColumnSchema, ColumnType, CreateDatasetRequest, Dataset, DatasetResponse, FileMeta,
        IngestTask,
    },
    DbPool,
};
use crate::services::{EmbeddingService, FileService};
use crate::utils::get_china_time;
use anyhow::{anyhow, Result};
use calamine::Reader;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use elasticsearch::http::transport::Transport;
use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::indices::IndicesRefreshParts;
use elasticsearch::{BulkParts, DeleteByQueryParts, Elasticsearch, SearchParts};
use serde_json::{json, Number, Value};
use sqlx::Row;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

const VECTOR_DIMS: usize = 1024; // 与现有ES向量维度保持一致
const BATCH_SIZE: usize = 1000; // ES bulk 批次大小（每批文档数量）

// —— 类型推断工具函数（模块级） ——
fn detect_type(value: &str) -> Option<ColumnType> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }

    // datetime formats
    let dt_formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d",
        "%Y/%m/%d",
    ];
    for f in dt_formats.iter() {
        if NaiveDateTime::parse_from_str(v, f).is_ok() {
            return Some(ColumnType::Datatime);
        }
        if NaiveDate::parse_from_str(v, f).is_ok() {
            return Some(ColumnType::Datatime);
        }
    }

    // integer
    if v.parse::<i64>().is_ok() {
        return Some(ColumnType::Long);
    }
    // float
    if v.parse::<f64>().is_ok() {
        return Some(ColumnType::Double);
    }
    // fallback string
    Some(ColumnType::String)
}

fn resolve_types(set: Option<&HashSet<ColumnType>>) -> (ColumnType, Option<String>) {
    match set {
        None => (ColumnType::String, None),
        Some(s) if s.is_empty() => (ColumnType::String, None),
        Some(s) => {
            if s.len() == 1 {
                return (s.iter().next().cloned().unwrap_or(ColumnType::String), None);
            }

            let has_long = s.contains(&ColumnType::Long);
            let has_double = s.contains(&ColumnType::Double);
            let has_dt = s.contains(&ColumnType::Datatime);
            let has_string = s.contains(&ColumnType::String);

            // Long + Double -> Double（且不含其他类型）
            if has_long && has_double && !has_dt && !has_string {
                return (ColumnType::Double, None);
            }

            // 存在 String 或 Datatime 与数字混杂，降级为 String，写入冲突信息
            let kinds: Vec<&'static str> = s
                .iter()
                .map(|t| match t {
                    ColumnType::String => "string",
                    ColumnType::Long => "long",
                    ColumnType::Double => "double",
                    ColumnType::Datatime => "datatime",
                })
                .collect();
            let msg = format!(
                "type conflict: detected [{}], default to string",
                kinds.join(",")
            );
            (ColumnType::String, Some(msg))
        }
    }
}

pub struct TableRagService {
    pool: DbPool,
    client: Elasticsearch,
    embedding_service: Arc<EmbeddingService>,
    file_service: Arc<FileService>,
}

impl TableRagService {
    pub async fn new(
        embedding_config: &EmbeddingConfig,
        embedding_service: Arc<EmbeddingService>,
        pool: DbPool,
        file_service: Arc<FileService>,
    ) -> Result<Self> {
        let es_cfg = embedding_config
            .elasticsearch
            .as_ref()
            .ok_or_else(|| anyhow!("Elasticsearch configuration not found"))?;
        let url = format!(
            r#"http://{}:{}@{}:{}"#,
            es_cfg.user, es_cfg.password, es_cfg.host, es_cfg.port
        );
        let transport = Transport::single_node(&url)?;
        let client = Elasticsearch::new(transport);
        if let Err(_) = client.ping().send().await {
            return Err(anyhow!("Elasticsearch connection error"));
        }

        let service = Self {
            pool,
            client,
            embedding_service,
            file_service,
        };
        // 按数据集独立索引维护，初始化无需创建全局索引
        service.init_schema().await?;
        Ok(service)
    }

    async fn init_schema(&self) -> Result<()> {
        // 服务启动时，扫描未完成/失败任务，清理对应ES数据并重新执行
        let unfinished_tasks: Vec<crate::models::table_rag::IngestTask> = sqlx::query_as(
            r#"SELECT id, dataset_id, file_id, status, error, create_time, update_time FROM t_task WHERE status != 2"#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        for task in unfinished_tasks.into_iter() {
            // 获取数据集索引
            if let Ok(dataset) = self.get_dataset_by_id(task.dataset_id).await {
                // 按 task_id 删除该任务写入的所有文档
                let _ = self
                    .client
                    .delete_by_query(DeleteByQueryParts::Index(&[&dataset.index_name]))
                    .body(json!({
                        "query": { "term": { "task_id": { "value": task.id.to_string() } } }
                    }))
                    .send()
                    .await;

                // 将任务重置为Created并重新执行
                let _ = sqlx::query(
                    r#"UPDATE t_task SET status = ?, error = NULL, update_time = ? WHERE id = ?"#,
                )
                .bind(0i32)
                .bind(crate::utils::get_china_time())
                .bind(task.id.to_string())
                .execute(&self.pool)
                .await;

                let service = Self {
                    pool: self.pool.clone(),
                    client: self.client.clone(),
                    embedding_service: self.embedding_service.clone(),
                    file_service: self.file_service.clone(),
                };
                tokio::spawn(async move {
                    if let Err(err) = service.run_ingest_task(task.id).await {
                        tracing::error!("restart recovery task failed: {}", err);
                    }
                });
            }
        }

        Ok(())
    }

    pub async fn create_dataset(&self, req: CreateDatasetRequest) -> Result<DatasetResponse> {
        let id = Uuid::new_v4();
        let now = get_china_time();

        let schema_value = serde_json::to_value(&req.schema)?;
        let schema_str = serde_json::to_string(&schema_value)?;

        let dtype = match req.r#type {
            crate::models::table_rag::DatasetType::Upload => "upload",
            crate::models::table_rag::DatasetType::Remote => "remote",
        };

        // 表名唯一性校验（避免重复）：规范化比较（去空格、忽略大小写）
        let normalized_name = req.name.trim();
        let normalized_table_name = req.table_name.trim();
        if let Ok(cnt) = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM t_dataset WHERE LOWER(TRIM(table_name)) = LOWER(TRIM(?))"#,
        )
        .bind(normalized_table_name)
        .fetch_one(&self.pool)
        .await
        {
            if cnt > 0 {
                return Err(anyhow!("Table name already exists"));
            }
        }

        // Generate ES index name per spec: datetime_uuid_vector (uuid without '-')
        let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let uid = Uuid::new_v4().to_string().replace('-', "");
        let index_name = format!("{}_{}_vector", ts, uid);

        sqlx::query(
            r#"INSERT INTO t_dataset (id, name, description, type, table_name, index_name, table_schema, retrieval_column, reply_column, similarity_threshold, max_results, create_time, update_time)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(id.to_string())
        .bind(&normalized_name)
        .bind(&req.description)
        .bind(dtype)
        .bind(&normalized_table_name)
        .bind(&index_name)
        .bind(schema_str)
        .bind(req.retrieval_column.as_deref().unwrap_or(""))
        .bind(req.reply_column.as_deref().unwrap_or(""))
        .bind(req.similarity_threshold.unwrap_or(0.3))
        .bind(req.max_results.unwrap_or(10))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let dataset = self.get_dataset_by_id(id).await?;
        Ok(dataset.into())
    }

    pub async fn list_datasets(&self) -> Result<Vec<DatasetResponse>> {
        let rows = sqlx::query_as::<_, Dataset>(
            r#"SELECT id, name, description, type, table_name, index_name, table_schema, index_mapping, retrieval_column, reply_column, similarity_threshold, max_results, create_time, update_time FROM t_dataset ORDER BY update_time DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|d| d.into()).collect())
    }

    pub async fn list_datasets_paged(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<DatasetResponse>> {
        let limit = page_size.max(1);
        let offset = (page.saturating_sub(1) * limit) as i64;
        let rows = sqlx::query_as::<_, Dataset>(
            r#"SELECT id, name, description, type, table_name, index_name, table_schema, index_mapping, retrieval_column, reply_column, similarity_threshold, max_results, create_time, update_time
               FROM t_dataset ORDER BY update_time DESC LIMIT ? OFFSET ?"#
        )
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|d| d.into()).collect())
    }

    pub async fn update_dataset(
        &self,
        id: Uuid,
        req: crate::models::table_rag::UpdateDatasetRequest,
    ) -> Result<DatasetResponse> {
        // Fetch current dataset
        let current = self.get_dataset_by_id(id).await?;

        // Compute new values (fallback to current if None)
        let new_name = req.name.unwrap_or(current.name.clone());
        let new_desc = match req.description {
            Some(d) => Some(d),
            None => current.description.clone(),
        };
        let new_retrieval = req
            .retrieval_column
            .unwrap_or_else(|| current.retrieval_column.clone());
        let new_reply = req
            .reply_column
            .unwrap_or_else(|| current.reply_column.clone());
        let new_sim = req
            .similarity_threshold
            .unwrap_or(current.similarity_threshold);
        let new_max = req.max_results.unwrap_or(current.max_results);
        let now = get_china_time();

        sqlx::query(
            r#"UPDATE t_dataset 
               SET name = ?, description = ?, retrieval_column = ?, reply_column = ?, similarity_threshold = ?, max_results = ?, update_time = ? 
               WHERE id = ?"#,
        )
        .bind(&new_name)
        .bind(&new_desc)
        .bind(&new_retrieval)
        .bind(&new_reply)
        .bind(new_sim)
        .bind(new_max)
        .bind(now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        let updated = self.get_dataset_by_id(id).await?;
        Ok(updated.into())
    }

    pub async fn preview_schema_from_files(
        &self,
        file_ids: Vec<Uuid>,
    ) -> Result<Vec<ColumnSchema>> {
        if file_ids.is_empty() {
            return Ok(vec![]);
        }

        // 汇总所有文件的表头及采样到的类型
        // 注意：需要保留字段首次出现顺序
        let mut headers_order: Vec<String> = Vec::new();
        let mut header_seen: HashSet<String> = HashSet::new();
        let mut observed_types: BTreeMap<String, HashSet<ColumnType>> = BTreeMap::new();
        let sample_rows: usize = 100;

        let mut register = |name: &str, value: &str| {
            let name = name.trim();
            if name.is_empty() {
                return;
            }
            if let Some(t) = detect_type(value) {
                observed_types
                    .entry(name.to_string())
                    .or_default()
                    .insert(t);
            }
        };

        for fid in file_ids {
            let file = self.get_file_by_id(fid).await?;
            match file.r#type.as_str() {
                "csv" => {
                    let bytes = self.file_service.read_by_path(&file.path).await?;
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .from_reader(Cursor::new(bytes));
                    let hs = rdr.headers()?.clone();
                    // 收集表头（保序）
                    for h in hs.iter() {
                        let name = h.trim();
                        if !name.is_empty() {
                            if header_seen.insert(name.to_string()) {
                                headers_order.push(name.to_string());
                            }
                        }
                    }
                    // 采样数据行进行类型推断
                    for (idx, result) in rdr.records().enumerate() {
                        if idx >= sample_rows {
                            break;
                        }
                        let record = result?;
                        for (i, h) in hs.iter().enumerate() {
                            let v = record.get(i).unwrap_or("");
                            if !v.trim().is_empty() {
                                register(h, v);
                            }
                        }
                    }
                }
                "excel" | "xlsx" => {
                    // 通过存储读取字节并写入临时文件，再用calamine读取
                    let bytes = self.file_service.read_by_path(&file.path).await?;
                    let tmp_path =
                        std::env::temp_dir().join(format!("mcp_tmp_{}.xlsx", Uuid::new_v4()));
                    fs::write(&tmp_path, &bytes)?;
                    let mut workbook = calamine::open_workbook_auto(&tmp_path)?;
                    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
                        let mut hs: Vec<String> = Vec::new();
                        for (r, row) in range.rows().enumerate() {
                            if r == 0 {
                                // 表头
                                hs = row
                                    .iter()
                                    .enumerate()
                                    .map(|(i, c)| {
                                        let v = c.to_string();
                                        let name = if v.trim().is_empty() {
                                            format!("col_{}", i)
                                        } else {
                                            v.trim().to_string()
                                        };
                                        if header_seen.insert(name.clone()) {
                                            headers_order.push(name.clone());
                                        }
                                        name
                                    })
                                    .collect();
                                continue;
                            }
                            if r > sample_rows {
                                break;
                            }
                            for (i, cell) in row.iter().enumerate() {
                                let h = hs.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
                                let v = cell.to_string();
                                if !v.trim().is_empty() {
                                    register(&h, &v);
                                }
                            }
                        }
                    }
                    let _ = fs::remove_file(&tmp_path);
                }
                other => return Err(anyhow!("Unsupported file type: {}", other)),
            }
        }

        // 合并类型并生成 schema
        let schema: Vec<ColumnSchema> = headers_order
            .into_iter()
            .map(|name| {
                let set = observed_types.get(&name);
                let (data_type, desc) = resolve_types(set);
                ColumnSchema {
                    name,
                    data_type,
                    description: desc,
                    searchable: true,
                    retrievable: true,
                }
            })
            .collect();

        Ok(schema)
    }

    pub async fn create_ingest_task(&self, dataset_id: Uuid, file_id: Uuid) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        let now = crate::utils::get_china_time();
        sqlx::query(r#"INSERT INTO t_task (id, dataset_id, file_id, status, error, create_time, update_time) VALUES (?, ?, ?, ?, ?, ?, ?)"#)
            .bind(task_id.to_string())
            .bind(dataset_id.to_string())
            .bind(file_id.to_string())
            .bind(0i32)
            .bind(Option::<String>::None)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(task_id)
    }

    pub async fn run_ingest_task(&self, task_id: Uuid) -> Result<u32> {
        // 读取任务信息
        let task = self.get_task_by_id(task_id).await?;
        // 标记 Processing
        sqlx::query(r#"UPDATE t_task SET status = ?, update_time = ? WHERE id = ?"#)
            .bind(1i32)
            .bind(crate::utils::get_china_time())
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await?;

        // 执行摄取（使用现有任务ID）
        match self
            .ingest_file_to_dataset(task_id, task.dataset_id, task.file_id)
            .await
        {
            Ok(rows) => {
                // 标记完成
                sqlx::query(r#"UPDATE t_task SET status = ?, update_time = ? WHERE id = ?"#)
                    .bind(2i32)
                    .bind(crate::utils::get_china_time())
                    .bind(task_id.to_string())
                    .execute(&self.pool)
                    .await?;
                Ok(rows)
            }
            Err(err) => {
                sqlx::query(
                    r#"UPDATE t_task SET status = ?, error = ?, update_time = ? WHERE id = ?"#,
                )
                .bind(3i32)
                .bind(err.to_string())
                .bind(crate::utils::get_china_time())
                .bind(task_id.to_string())
                .execute(&self.pool)
                .await?;
                Err(err)
            }
        }
    }

    async fn get_task_by_id(&self, id: Uuid) -> Result<crate::models::table_rag::IngestTask> {
        let row = sqlx::query_as::<_, crate::models::table_rag::IngestTask>(
            r#"SELECT id, dataset_id, file_id, status, error, create_time, update_time FROM t_task WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_tasks_by_dataset(
        &self,
        dataset_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<IngestTask>> {
        let limit = page_size.max(1);
        let offset = (page.saturating_sub(1) * limit) as i64;
        let rows = sqlx::query_as::<_, IngestTask>(
            r#"SELECT id, dataset_id, file_id, status, error, create_time, update_time
               FROM t_task WHERE dataset_id = ? ORDER BY create_time DESC LIMIT ? OFFSET ?"#,
        )
        .bind(dataset_id.to_string())
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // 远程数据库支持：MySQL
    pub async fn test_remote_connection_mysql(&self, url: &str) -> Result<()> {
        let pool = sqlx::MySqlPool::connect(url).await?;
        let _version: (String,) = sqlx::query_as("SELECT VERSION()").fetch_one(&pool).await?;
        Ok(())
    }

    pub async fn list_remote_tables_mysql(&self, url: &str) -> Result<Vec<String>> {
        let pool = sqlx::MySqlPool::connect(url).await?;
        // 读取当前数据库下的表名
        let rows = sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE() ORDER BY table_name")
            .fetch_all(&pool)
            .await?;
        let mut tables = Vec::new();
        for row in rows {
            if let Ok(name) = row.try_get::<String, _>("table_name") {
                tables.push(name);
            }
        }
        Ok(tables)
    }

    pub async fn ingest_file_to_dataset(
        &self,
        task_id: Uuid,
        dataset_id: Uuid,
        file_id: Uuid,
    ) -> Result<u32> {
        let dataset = self.get_dataset_by_id(dataset_id).await?;
        let file = self.get_file_by_id(file_id).await?;

        // 解析表schema，找出searchable列
        let columns: Vec<ColumnSchema> =
            serde_json::from_value(dataset.table_schema.clone()).unwrap_or_default();
        // Use retrieval_column if configured; otherwise fallback to schema.searchable
        let searchable: Vec<String> = {
            let rc = dataset.retrieval_column.trim();
            if !rc.is_empty() {
                rc.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                columns
                    .iter()
                    .filter(|c| c.searchable)
                    .map(|c| c.name.clone())
                    .collect()
            }
        };
        let schema_columns_set: HashSet<String> = columns.iter().map(|c| c.name.clone()).collect();

        // 使用传入的现有 task_id，不再新建任务记录
        // 标记 Processing
        sqlx::query(r#"UPDATE t_task SET status = ?, update_time = ? WHERE id = ?"#)
            .bind(1i32)
            .bind(get_china_time())
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await?;

        // 创建数据集独立索引（若不存在）并按 0055 规范设置 mapping
        self.ensure_dataset_index(&dataset, &columns).await?;

        let mut body: Vec<String> = Vec::new();
        let mut total_rows: u32 = 0;

        match file.r#type.as_str() {
            "csv" => {
                let bytes = self.file_service.read_by_path(&file.path).await?;
                let mut rdr = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .from_reader(Cursor::new(bytes));
                let headers = rdr.headers()?.clone();
                // 校验文件头与知识库schema一致（忽略顺序）
                let header_set: HashSet<String> = headers.iter().map(|s| s.to_string()).collect();
                if header_set != schema_columns_set {
                    let diff_desc = format!(
                        "schema mismatch: dataset={{{:?}}} file={{{:?}}}",
                        schema_columns_set, header_set
                    );
                    // 标记任务失败
                    sqlx::query(
                        r#"UPDATE t_task SET status = ?, error = ?, update_time = ? WHERE id = ?"#,
                    )
                    .bind(3i32)
                    .bind(diff_desc)
                    .bind(get_china_time())
                    .bind(task_id.to_string())
                    .execute(&self.pool)
                    .await?;
                    return Err(anyhow!("File headers do not match dataset schema"));
                }
                for result in rdr.records() {
                    let record = result?;
                    let mut doc_fields = serde_json::Map::new();
                    let mut text_parts: Vec<String> = Vec::new();
                    for (i, h) in headers.iter().enumerate() {
                        let v = record.get(i).unwrap_or("");
                        // 类型转换依据 ColumnSchema
                        let ty = columns.iter().find(|c| c.name == h).map(|c| &c.data_type);
                        match ty {
                            Some(ColumnType::Long) => {
                                if let Ok(n) = v.parse::<i64>() {
                                    doc_fields
                                        .insert(h.to_string(), Value::Number(Number::from(n)));
                                } else {
                                    doc_fields.insert(h.to_string(), Value::String(v.to_string()));
                                }
                            }
                            Some(ColumnType::Double) => {
                                if let Ok(f) = v.parse::<f64>() {
                                    if let Some(num) = Number::from_f64(f) {
                                        doc_fields.insert(h.to_string(), Value::Number(num));
                                    } else {
                                        doc_fields
                                            .insert(h.to_string(), Value::String(v.to_string()));
                                    }
                                } else {
                                    doc_fields.insert(h.to_string(), Value::String(v.to_string()));
                                }
                            }
                            Some(ColumnType::Datatime) => {
                                doc_fields.insert(h.to_string(), Value::String(v.to_string()));
                            }
                            _ => {
                                doc_fields.insert(h.to_string(), Value::String(v.to_string()));
                            }
                        }
                        if searchable.contains(&h.to_string()) {
                            text_parts.push(format!("{}:{}", h, v));
                        }
                    }
                    let text = text_parts.join(" \n\n ");
                    let embedding = self.embedding_service.embed_text(&text).await?;

                    body.push(json!({"index": {"_index": dataset.index_name, "_id": Uuid::new_v4().to_string()}}).to_string());
                    let mut doc = serde_json::Map::new();
                    doc.insert(
                        "file_name".to_string(),
                        Value::String(file.name.clone().unwrap_or_default()),
                    );
                    doc.insert("sheet".to_string(), Value::String(String::new())); // CSV 无 sheet
                                                                                   // row_vector: 直接写入向量
                    doc.insert(
                        "row_vector".to_string(),
                        Value::Array(
                            embedding
                                .into_iter()
                                .map(|v| Number::from_f64(v as f64).map(Value::Number).unwrap())
                                .collect(),
                        ),
                    );
                    // 列值展平到根
                    for (k, v) in doc_fields.into_iter() {
                        doc.insert(k, v);
                    }
                    body.push(Value::Object(doc).to_string());
                    total_rows += 1;
                    // 每批次提交一次 bulk
                    if (total_rows as usize) % BATCH_SIZE == 0 {
                        let batch = std::mem::take(&mut body);
                        let _ = self
                            .client
                            .bulk(BulkParts::Index(&dataset.index_name))
                            .body(batch)
                            .send()
                            .await?;
                    }
                }
            }
            "excel" | "xlsx" => {
                let bytes = self.file_service.read_by_path(&file.path).await?;
                let tmp_path =
                    std::env::temp_dir().join(format!("mcp_tmp_{}.xlsx", Uuid::new_v4()));
                fs::write(&tmp_path, &bytes)?;
                let mut workbook = calamine::open_workbook_auto(&tmp_path)?;
                let range = workbook
                    .worksheet_range_at(0)
                    .ok_or_else(|| anyhow!("No sheet found"))??;
                let sheet_name = workbook
                    .sheet_names()
                    .get(0)
                    .cloned()
                    .unwrap_or_else(|| "".to_string());
                let mut headers: Vec<String> = Vec::new();
                for (r, row) in range.rows().enumerate() {
                    if r == 0 {
                        headers = row.iter().map(|c| c.to_string()).collect();
                        // 校验文件头与知识库schema一致（忽略顺序）
                        let header_set: HashSet<String> = headers.iter().cloned().collect();
                        if header_set != schema_columns_set {
                            let diff_desc = format!(
                                "schema mismatch: dataset={{{:?}}} file={{{:?}}}",
                                schema_columns_set, header_set
                            );
                            sqlx::query(r#"UPDATE t_task SET status = ?, error = ?, update_time = ? WHERE id = ?"#)
                                .bind(3i32)
                                .bind(diff_desc)
                                .bind(crate::utils::get_china_time())
                                .bind(task_id.to_string())
                                .execute(&self.pool)
                                .await?;
                            let _ = fs::remove_file(&tmp_path);
                            return Err(anyhow!("File headers do not match dataset schema"));
                        }
                        continue;
                    }
                    let mut doc_fields = serde_json::Map::new();
                    let mut text_parts: Vec<String> = Vec::new();
                    for (i, cell) in row.iter().enumerate() {
                        let h = headers
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("col_{}", i));
                        let v = cell.to_string();
                        let ty = columns.iter().find(|c| c.name == h).map(|c| &c.data_type);
                        match ty {
                            Some(ColumnType::Long) => {
                                if let Ok(n) = v.parse::<i64>() {
                                    doc_fields.insert(h.clone(), Value::Number(Number::from(n)));
                                } else {
                                    doc_fields.insert(h.clone(), Value::String(v.clone()));
                                }
                            }
                            Some(ColumnType::Double) => {
                                if let Ok(f) = v.parse::<f64>() {
                                    if let Some(num) = Number::from_f64(f) {
                                        doc_fields.insert(h.clone(), Value::Number(num));
                                    } else {
                                        doc_fields.insert(h.clone(), Value::String(v.clone()));
                                    }
                                } else {
                                    doc_fields.insert(h.clone(), Value::String(v.clone()));
                                }
                            }
                            Some(ColumnType::Datatime) => {
                                doc_fields.insert(h.clone(), Value::String(v.clone()));
                            }
                            _ => {
                                doc_fields.insert(h.clone(), Value::String(v.clone()));
                            }
                        }
                        if searchable.contains(&h) {
                            text_parts.push(format!("{}:{}", h, v));
                        }
                    }
                    let text = text_parts.join(" \n\n ");
                    tracing::debug!("embed text: {}", text);
                    let embedding = self.embedding_service.embed_text(&text).await?;
                    body.push(json!({"index": {"_index": dataset.index_name, "_id": Uuid::new_v4().to_string()}}).to_string());
                    let mut doc = serde_json::Map::new();
                    doc.insert(
                        "file_name".to_string(),
                        Value::String(file.name.clone().unwrap_or_default()),
                    );
                    doc.insert("sheet".to_string(), Value::String(sheet_name.clone()));
                    doc.insert(
                        "row_vector".to_string(),
                        Value::Array(
                            embedding
                                .into_iter()
                                .map(|v| Number::from_f64(v as f64).map(Value::Number).unwrap())
                                .collect(),
                        ),
                    );
                    // 绑定任务ID，便于重启清理
                    doc.insert("task_id".to_string(), Value::String(task_id.to_string()));
                    for (k, v) in doc_fields.into_iter() {
                        doc.insert(k, v);
                    }
                    body.push(Value::Object(doc).to_string());
                    total_rows += 1;
                    if (total_rows as usize) % BATCH_SIZE == 0 {
                        let batch = std::mem::take(&mut body);
                        let _ = self
                            .client
                            .bulk(BulkParts::Index(&dataset.index_name))
                            .body(batch)
                            .send()
                            .await?;
                    }
                }
                let _ = fs::remove_file(&tmp_path);
            }
            other => {
                return Err(anyhow!("Unsupported file type: {}", other));
            }
        }

        if !body.is_empty() {
            let _ = self
                .client
                .bulk(BulkParts::Index(&dataset.index_name))
                .body(body)
                .send()
                .await?;
        }
        let _ = self
            .client
            .indices()
            .refresh(IndicesRefreshParts::Index(&[&dataset.index_name]))
            .send()
            .await?;

        // 写入 dataset-file 映射
        let df_id = Uuid::new_v4();
        let _ = sqlx::query(
            r#"INSERT IGNORE INTO t_dataset_file (id, dataset_id, file_id) VALUES (?, ?, ?)"#,
        )
        .bind(df_id.to_string())
        .bind(dataset.id.to_string())
        .bind(file.id.to_string())
        .execute(&self.pool)
        .await?;

        // 状态更新由 run_ingest_task 负责，这里不更新任务状态

        Ok(total_rows)
    }

    pub async fn search(
        &self,
        dataset_id: Uuid,
        query: &str,
        max_results: u32,
        similarity_threshold: Option<f32>,
    ) -> Result<Value> {
        let dataset = self.get_dataset_by_id(dataset_id).await?;
        // 默认返回数量：当未显式传入或为0时，使用数据集配置的默认值
        let max_results = if max_results == 0 {
            dataset.max_results as u32
        } else {
            max_results
        };
        let query_embedding = self
            .embedding_service
            .embed_text(query)
            .await?
            .into_iter()
            .map(|v| Value::Number(Number::from_f64(v as f64).unwrap()))
            .collect::<Vec<Value>>();

        let mut knn = serde_json::map::Map::new();
        knn.insert("field".to_string(), Value::String("row_vector".to_string()));
        knn.insert("query_vector".to_string(), Value::Array(query_embedding));
        knn.insert("k".to_string(), Value::Number(Number::from(max_results)));
        knn.insert(
            "num_candidates".to_string(),
            Value::Number(Number::from(10000)),
        );

        // Limit returned fields to reply_column (comma-separated). If empty, default to all.
        let reply_cols: Vec<String> = dataset
            .reply_column
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut root = serde_json::map::Map::new();
        root.insert("knn".to_string(), Value::Object(knn));
        if !reply_cols.is_empty() {
            root.insert("_source".to_string(), json!({"includes": reply_cols}));
        } else {
            root.insert("_source".to_string(), Value::Bool(true));
        }
        root.insert("size".to_string(), Value::Number(Number::from(max_results)));

        let search_response = self
            .client
            .search(SearchParts::Index(&[&dataset.index_name]))
            .body(Value::Object(root))
            .send()
            .await?;
        let mut response_body = search_response.json::<Value>().await?;

        // 应用相似度阈值过滤：当未显式传入时，使用数据集默认值
        let effective_threshold = similarity_threshold.unwrap_or(dataset.similarity_threshold);
        if effective_threshold > 0.0 {
            if let Some(hits) = response_body["hits"]["hits"].as_array_mut() {
                hits.retain(|h| h["_score"].as_f64().unwrap_or(0.0) >= effective_threshold as f64);
            }
        }

        Ok(response_body)
    }

    pub async fn get_dataset_by_id(&self, id: Uuid) -> Result<Dataset> {
        let row = sqlx::query_as::<_, Dataset>(
            r#"SELECT id, name, description, type, table_name, index_name, table_schema, index_mapping, retrieval_column, reply_column, similarity_threshold, max_results, create_time, update_time FROM t_dataset WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn ensure_dataset_index(
        &self,
        dataset: &Dataset,
        columns: &Vec<ColumnSchema>,
    ) -> Result<()> {
        // 尝试创建索引（若存在，ES返回错误可忽略）
        let mut props = serde_json::Map::new();
        props.insert("file_name".to_string(), json!({"type":"keyword"}));
        props.insert("sheet".to_string(), json!({"type":"keyword"}));
        props.insert(
            "row_vector".to_string(),
            json!({"type":"dense_vector","dims": VECTOR_DIMS}),
        );
        // 添加 task_id 字段，便于任务级别清理
        props.insert("task_id".to_string(), json!({"type":"keyword"}));
        for c in columns {
            let v = match c.data_type {
                ColumnType::String => json!({"type":"text"}),
                ColumnType::Long => json!({"type":"long"}),
                ColumnType::Double => json!({"type":"double"}),
                ColumnType::Datatime => json!({"type":"date","format":"yyyy-MM-dd HH:mm:ss"}),
            };
            props.insert(c.name.clone(), v);
        }
        let body = json!({
            "mappings": { "properties": Value::Object(props) }
        });
        let _ = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(&dataset.index_name))
            .body(body.clone())
            .send()
            .await;
        // 保存 mapping 到数据库
        let mapping_str = serde_json::to_string(&body)?;
        let now = get_china_time();
        let _ =
            sqlx::query(r#"UPDATE t_dataset SET index_mapping = ?, update_time = ? WHERE id = ?"#)
                .bind(mapping_str)
                .bind(now)
                .bind(dataset.id.to_string())
                .execute(&self.pool)
                .await?;
        Ok(())
    }

    async fn get_file_by_id(&self, id: Uuid) -> Result<FileMeta> {
        let row = sqlx::query_as::<_, FileMeta>(
            r#"SELECT id, type, name, path, size, create_time, update_time FROM t_file WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
