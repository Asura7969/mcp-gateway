use sqlx::{MySql, MySqlPool, Pool};

pub type DbPool = Pool<MySql>;

pub async fn create_pool(database_url: &str, _max_connections: u32) -> Result<DbPool, sqlx::Error> {
    let pool = MySqlPool::connect(database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

use std::sync::OnceLock;

pub static DB_POOL: OnceLock<DbPool> = OnceLock::new();
