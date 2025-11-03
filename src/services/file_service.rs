use crate::config::{AliyunOssConfig, LocalStorageConfig, StorageConfig, StorageProvider};
use crate::models::table_rag::FileMeta;
use crate::models::DbPool;
use crate::utils::get_china_time;
use anyhow::Result;
use opendal::Operator;
use uuid::Uuid;

pub struct FileService {
    pool: DbPool,
    operator: Operator,
    root: String,
}

impl FileService {
    pub fn new(pool: DbPool, storage: Option<StorageConfig>) -> Result<Self> {
        let (operator, root) = if let Some(cfg) = storage {
            match cfg.provider {
                StorageProvider::Oss => {
                    let oss_cfg: AliyunOssConfig = cfg
                        .oss
                        .ok_or_else(|| anyhow::anyhow!("OSS storage config missing"))?;
                    let mut builder = opendal::services::Oss::default();
                    // Configure Aliyun OSS
                    if let Some(root) = oss_cfg.root.clone() {
                        builder.root(&root);
                    }
                    builder.endpoint(&oss_cfg.endpoint);
                    builder.bucket(&oss_cfg.bucket);
                    builder.access_key_id(&oss_cfg.access_key_id);
                    builder.access_key_secret(&oss_cfg.access_key_secret);
                    let operator = Operator::new(builder)?.finish();
                    let root = oss_cfg.root.unwrap_or_else(|| "table_rag".to_string());
                    (operator, root)
                }
                StorageProvider::Local => {
                    let local_cfg: LocalStorageConfig = cfg.local.unwrap_or(LocalStorageConfig {
                        root: "storage/uploads".to_string(),
                    });
                    let mut builder = opendal::services::Fs::default();
                    builder.root(&local_cfg.root);
                    let operator = Operator::new(builder)?.finish();
                    (operator, local_cfg.root)
                }
            }
        } else {
            // Default to local filesystem storage
            let mut builder = opendal::services::Fs::default();
            builder.root("storage/uploads");
            let operator = Operator::new(builder)?.finish();
            (operator, "storage/uploads".to_string())
        };

        Ok(Self {
            pool,
            operator,
            root,
        })
    }

    pub async fn upload_and_save(&self, filename: &str, bytes: Vec<u8>) -> Result<FileMeta> {
        let id = Uuid::new_v4();
        let now = get_china_time();

        let ext = filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let ftype = if ext == "csv" {
            "csv"
        } else if ext == "xlsx" || ext == "xls" {
            "excel"
        } else {
            ext.as_str()
        };

        // Object key under configured root
        let key = format!("{}/{}", id, filename);

        // Write to storage
        let size = bytes.len() as i64;
        self.operator.write(&key, bytes).await?;

        // Insert metadata
        sqlx::query(
            r#"INSERT INTO t_file (id, type, name, path, size, create_time, update_time) VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(id.to_string())
        .bind(ftype)
        .bind(filename)
        .bind(&format!("{}/{}", self.root, key))
        .bind(size)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Build response
        let row = sqlx::query_as::<_, FileMeta>(
            r#"SELECT id, type, name, path, size, create_time, update_time FROM t_file WHERE id = ?"#
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Read file content by stored path value (compatible with local/OSS).
    pub async fn read_by_path(&self, path: &str) -> Result<Vec<u8>> {
        // Stored path is like "{root}/{id}/{filename}". Convert to operator key.
        let root = self.root.trim_end_matches('/');
        let prefix = format!("{}/", root);
        let key = path.strip_prefix(&prefix).unwrap_or(path);
        let data = self.operator.read(key).await?;
        Ok(data)
    }
}
