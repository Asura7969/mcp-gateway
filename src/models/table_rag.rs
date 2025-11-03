use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatasetType {
    Upload,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    String,
    Long,
    Double,
    Datatime, // yyyy-MM-dd HH:mm:ss
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: ColumnType,
    pub description: Option<String>,
    /// 参与检索：是否作为检索字段
    #[serde(default)]
    pub searchable: bool,
    /// 参与回复：是否在命中后作为上下文返回
    #[serde(default)]
    pub retrievable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    #[serde(with = "uuid_as_string")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: DatasetType,
    pub table_name: String,
    pub index_name: String,
    pub table_schema: serde_json::Value,
    #[serde(default)]
    pub index_mapping: Option<serde_json::Value>,
    #[serde(default)]
    pub retrieval_column: String,
    #[serde(default)]
    pub reply_column: String,
    pub similarity_threshold: f32,
    pub max_results: i32,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl FromRow<'_, MySqlRow> for Dataset {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let type_str: String = row.try_get("type")?;
        let dtype = match type_str.as_str() {
            "upload" => DatasetType::Upload,
            "remote" => DatasetType::Remote,
            _ => DatasetType::Upload,
        };
        let schema_str: String = row.try_get("table_schema")?;
        let table_schema: serde_json::Value = serde_json::from_str(&schema_str)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid JSON: {}", e).into()))?;
        let index_mapping: Option<serde_json::Value> =
            match row.try_get::<Option<String>, _>("index_mapping")? {
                Some(s) => serde_json::from_str(&s).ok(),
                None => None,
            };

        Ok(Self {
            id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            r#type: dtype,
            table_name: row.try_get("table_name")?,
            index_name: row.try_get("index_name")?,
            table_schema,
            index_mapping,
            retrieval_column: row.try_get("retrieval_column").unwrap_or_default(),
            reply_column: row.try_get("reply_column").unwrap_or_default(),
            similarity_threshold: row.try_get::<f32, _>("similarity_threshold")?,
            max_results: row.try_get::<i32, _>("max_results")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    #[serde(with = "uuid_as_string")]
    pub id: Uuid,
    pub r#type: String, // csv, excel
    pub name: Option<String>,
    pub path: String, // oss路径或本地存储路径
    pub size: Option<i64>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl FromRow<'_, MySqlRow> for FileMeta {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        Ok(Self {
            id,
            r#type: row.try_get("type")?,
            name: row.try_get("name")?,
            path: row.try_get("path")?,
            size: row.try_get("size")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetFileMap {
    #[serde(with = "uuid_as_string")]
    pub id: Uuid,
    #[serde(with = "uuid_as_string")]
    pub dataset_id: Uuid,
    #[serde(with = "uuid_as_string")]
    pub file_id: Uuid,
}

impl FromRow<'_, MySqlRow> for DatasetFileMap {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id = Uuid::parse_str(&row.try_get::<String, _>("id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let dataset_id = Uuid::parse_str(&row.try_get::<String, _>("dataset_id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let file_id = Uuid::parse_str(&row.try_get::<String, _>("file_id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        Ok(Self {
            id,
            dataset_id,
            file_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Created = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
}

impl From<i32> for TaskStatus {
    fn from(v: i32) -> Self {
        match v {
            1 => TaskStatus::Processing,
            2 => TaskStatus::Completed,
            3 => TaskStatus::Failed,
            _ => TaskStatus::Created,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestTask {
    #[serde(with = "uuid_as_string")]
    pub id: Uuid,
    #[serde(with = "uuid_as_string")]
    pub dataset_id: Uuid,
    #[serde(with = "uuid_as_string")]
    pub file_id: Uuid,
    pub status: TaskStatus,
    pub error: Option<String>,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl FromRow<'_, MySqlRow> for IngestTask {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let id = Uuid::parse_str(&row.try_get::<String, _>("id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let dataset_id = Uuid::parse_str(&row.try_get::<String, _>("dataset_id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let file_id = Uuid::parse_str(&row.try_get::<String, _>("file_id")?)
            .map_err(|e| sqlx::Error::Decode(format!("Invalid UUID: {}", e).into()))?;
        let status = TaskStatus::from(row.try_get::<i32, _>("status")?);
        Ok(Self {
            id,
            dataset_id,
            file_id,
            status,
            error: row.try_get("error")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: Option<String>,
    pub r#type: DatasetType,
    pub table_name: String,
    pub schema: Vec<ColumnSchema>,
    pub similarity_threshold: Option<f32>,
    pub max_results: Option<i32>,
    #[serde(default)]
    pub retrieval_column: Option<String>,
    #[serde(default)]
    pub reply_column: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDatasetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub similarity_threshold: Option<f32>,
    pub max_results: Option<i32>,
    #[serde(default)]
    pub retrieval_column: Option<String>,
    #[serde(default)]
    pub reply_column: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: DatasetType,
    pub table_name: String,
    pub similarity_threshold: f32,
    pub max_results: i32,
}

impl From<Dataset> for DatasetResponse {
    fn from(d: Dataset) -> Self {
        Self {
            id: d.id,
            name: d.name,
            description: d.description,
            r#type: d.r#type,
            table_name: d.table_name,
            similarity_threshold: d.similarity_threshold,
            max_results: d.max_results,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: DatasetType,
    pub table_name: String,
    pub index_name: String,
    pub table_schema: Vec<ColumnSchema>,
    pub index_mapping: Option<serde_json::Value>,
    pub retrieval_column: String,
    pub reply_column: String,
    pub similarity_threshold: f32,
    pub max_results: i32,
}

impl From<Dataset> for DatasetDetailResponse {
    fn from(d: Dataset) -> Self {
        let table_schema: Vec<ColumnSchema> =
            serde_json::from_value(d.table_schema.clone()).unwrap_or_default();
        Self {
            id: d.id,
            name: d.name,
            description: d.description,
            r#type: d.r#type,
            table_name: d.table_name,
            index_name: d.index_name,
            table_schema,
            index_mapping: d.index_mapping,
            retrieval_column: d.retrieval_column,
            reply_column: d.reply_column,
            similarity_threshold: d.similarity_threshold,
            max_results: d.max_results,
        }
    }
}

// Custom UUID (de)serialization helpers
mod uuid_as_string {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}
