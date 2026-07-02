use crate::connection::config::{ConnectionConfig, Driver};
use crate::dialect::connect_url;
use crate::error::AppError;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS: u32 = 5;

#[derive(Clone)]
pub enum DbPool {
    Postgres(sqlx::PgPool),
    MySql(sqlx::MySqlPool),
}

impl DbPool {
    pub fn driver(&self) -> Driver {
        match self {
            DbPool::Postgres(_) => Driver::Postgres,
            DbPool::MySql(_) => Driver::MySql,
        }
    }

    pub async fn close(&self) {
        match self {
            DbPool::Postgres(p) => p.close().await,
            DbPool::MySql(p) => p.close().await,
        }
    }
}

pub async fn connect(config: &ConnectionConfig, password: &str) -> Result<DbPool, AppError> {
    let url = connect_url(config, password);
    let connect = async {
        match config.driver {
            Driver::Postgres => PgPoolOptions::new()
                .max_connections(MAX_CONNECTIONS)
                .acquire_timeout(CONNECT_TIMEOUT)
                .connect(&url)
                .await
                .map(DbPool::Postgres),
            Driver::MySql => MySqlPoolOptions::new()
                .max_connections(MAX_CONNECTIONS)
                .acquire_timeout(CONNECT_TIMEOUT)
                .connect(&url)
                .await
                .map(DbPool::MySql),
        }
    };
    tokio::time::timeout(CONNECT_TIMEOUT, connect)
        .await
        .map_err(|_| AppError::Timeout("connection timed out after 10s".into()))?
        .map_err(|e| AppError::Connection(e.to_string()))
}

pub async fn test_connection(config: &ConnectionConfig, password: &str) -> Result<(), AppError> {
    let pool = connect(config, password).await?;
    let result = match &pool {
        DbPool::Postgres(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
        DbPool::MySql(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
    };
    pool.close().await;
    result.map_err(|e| AppError::Connection(e.to_string()))
}

pub struct PoolManager {
    pools: RwLock<HashMap<String, DbPool>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self { pools: RwLock::new(HashMap::new()) }
    }

    pub async fn get_or_create(&self, config: &ConnectionConfig, password: &str) -> Result<DbPool, AppError> {
        if let Some(pool) = self.pools.read().await.get(&config.id) {
            return Ok(pool.clone());
        }
        let pool = connect(config, password).await?;
        self.pools.write().await.insert(config.id.clone(), pool.clone());
        Ok(pool)
    }

    pub async fn remove(&self, conn_id: &str) {
        if let Some(pool) = self.pools.write().await.remove(conn_id) {
            pool.close().await;
        }
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}
