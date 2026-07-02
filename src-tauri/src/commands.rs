use crate::connection::config::{ConfigStore, ConnectionConfig};
use crate::connection::pool::{test_connection, DbPool, PoolManager};
use crate::connection::secret;
use crate::error::AppError;
use crate::query::{self, QueryResult, TablePage, TablePageRequest};
use crate::safety;
use crate::schema::{self, ColumnInfo, TableInfo};
use tauri::State;

pub struct AppState {
    pub store: ConfigStore,
    pub pools: PoolManager,
}

impl AppState {
    async fn pool(&self, conn_id: &str) -> Result<DbPool, AppError> {
        let config = self
            .store
            .list()?
            .into_iter()
            .find(|c| c.id == conn_id)
            .ok_or_else(|| AppError::NotFound(format!("connection '{conn_id}'")))?;
        let password = secret::get_password(conn_id)?.unwrap_or_default();
        self.pools.get_or_create(&config, &password).await
    }
}

#[tauri::command]
pub async fn connections_list(state: State<'_, AppState>) -> Result<Vec<ConnectionConfig>, AppError> {
    state.store.list()
}

#[tauri::command]
pub async fn connection_save(
    state: State<'_, AppState>,
    config: ConnectionConfig,
    password: Option<String>,
) -> Result<(), AppError> {
    if let Some(p) = password {
        secret::set_password(&config.id, &p)?;
    }
    // 配置可能变更（host、密码等），丢弃旧池让下次重建
    state.pools.remove(&config.id).await;
    state.store.save(config)
}

#[tauri::command]
pub async fn connection_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.pools.remove(&id).await;
    secret::delete_password(&id)?;
    state.store.delete(&id)
}

#[tauri::command]
pub async fn connection_test(
    config: ConnectionConfig,
    password: Option<String>,
) -> Result<(), AppError> {
    // 编辑已有连接但未重输密码时，回退 Keychain
    let password = match password {
        Some(p) => p,
        None => secret::get_password(&config.id)?.unwrap_or_default(),
    };
    test_connection(&config, &password).await
}

#[tauri::command]
pub async fn list_namespaces(state: State<'_, AppState>, conn_id: String) -> Result<Vec<String>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_namespaces(&pool).await
}

#[tauri::command]
pub async fn list_tables(
    state: State<'_, AppState>,
    conn_id: String,
    namespace: String,
) -> Result<Vec<TableInfo>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_tables(&pool, &namespace).await
}

#[tauri::command]
pub async fn list_columns(
    state: State<'_, AppState>,
    conn_id: String,
    namespace: String,
    table: String,
) -> Result<Vec<ColumnInfo>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_columns(&pool, &namespace, &table).await
}

#[tauri::command]
pub async fn fetch_table_page(
    state: State<'_, AppState>,
    conn_id: String,
    req: TablePageRequest,
) -> Result<TablePage, AppError> {
    let pool = state.pool(&conn_id).await?;
    query::fetch_table_page(&pool, &req).await
}

#[tauri::command]
pub async fn execute_sql(
    state: State<'_, AppState>,
    conn_id: String,
    sql: String,
    confirmed: bool,
) -> Result<QueryResult, AppError> {
    if !confirmed {
        let warnings = safety::analyze(&sql);
        if let Some(w) = warnings.first() {
            let label = match w.kind {
                safety::DangerKind::DropDatabase => "DROP DATABASE",
                safety::DangerKind::DropTable => "DROP TABLE",
                safety::DangerKind::Truncate => "TRUNCATE",
                safety::DangerKind::DeleteWithoutWhere => "DELETE without WHERE",
                safety::DangerKind::UpdateWithoutWhere => "UPDATE without WHERE",
            };
            return Err(AppError::DangerousStatement(format!("{label}: {}", w.statement)));
        }
    }
    let pool = state.pool(&conn_id).await?;
    query::execute_sql(&pool, &sql).await
}
