use crate::connection::pool::DbPool;
use crate::error::AppError;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TableKind {
    Table,
    View,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
    pub kind: TableKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

pub async fn list_namespaces(pool: &DbPool) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = match pool {
        DbPool::Postgres(p) => {
            sqlx::query_scalar(
                "SELECT schema_name FROM information_schema.schemata
                 WHERE schema_name NOT IN ('information_schema') AND schema_name NOT LIKE 'pg_%'
                 ORDER BY schema_name",
            )
            .fetch_all(p)
            .await?
        }
        DbPool::MySql(p) => {
            sqlx::query_scalar(
                "SELECT CAST(schema_name AS CHAR) FROM information_schema.schemata
                 WHERE schema_name NOT IN ('mysql', 'information_schema', 'performance_schema', 'sys')
                 ORDER BY schema_name",
            )
            .fetch_all(p)
            .await?
        }
    };
    Ok(rows)
}

const TABLES_SQL_PG: &str = "SELECT table_name, table_type FROM information_schema.tables
    WHERE table_schema = $1 ORDER BY table_name";
const TABLES_SQL_MYSQL: &str = "SELECT CAST(table_name AS CHAR), CAST(table_type AS CHAR)
    FROM information_schema.tables
    WHERE table_schema = ? ORDER BY table_name";

pub async fn list_tables(pool: &DbPool, namespace: &str) -> Result<Vec<TableInfo>, AppError> {
    let raw: Vec<(String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(TABLES_SQL_PG).bind(namespace).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(TABLES_SQL_MYSQL).bind(namespace).fetch_all(p).await?,
    };
    Ok(raw
        .into_iter()
        .map(|(name, table_type)| TableInfo {
            name,
            kind: if table_type.to_uppercase().contains("VIEW") { TableKind::View } else { TableKind::Table },
        })
        .collect())
}

const PK_SQL_PG: &str = "SELECT kcu.column_name, c.data_type
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
      ON kcu.constraint_name = tc.constraint_name
     AND kcu.table_schema = tc.table_schema
     AND kcu.table_name = tc.table_name
    JOIN information_schema.columns c
      ON c.table_schema = kcu.table_schema
     AND c.table_name = kcu.table_name
     AND c.column_name = kcu.column_name
    WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = $1 AND tc.table_name = $2
    ORDER BY kcu.ordinal_position";
const PK_SQL_MYSQL: &str = "SELECT CAST(kcu.column_name AS CHAR), CAST(c.data_type AS CHAR)
    FROM information_schema.key_column_usage kcu
    JOIN information_schema.columns c
      ON c.table_schema = kcu.table_schema
     AND c.table_name = kcu.table_name
     AND c.column_name = kcu.column_name
    WHERE kcu.constraint_name = 'PRIMARY' AND kcu.table_schema = ? AND kcu.table_name = ?
    ORDER BY kcu.ordinal_position";

async fn primary_key_with_types(pool: &DbPool, namespace: &str, table: &str) -> Result<Vec<(String, String)>, AppError> {
    let rows: Vec<(String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(PK_SQL_PG).bind(namespace).bind(table).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(PK_SQL_MYSQL).bind(namespace).bind(table).fetch_all(p).await?,
    };
    Ok(rows)
}

const COLUMNS_SQL_PG: &str = "SELECT column_name, data_type, is_nullable
    FROM information_schema.columns
    WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position";
const COLUMNS_SQL_MYSQL: &str = "SELECT CAST(column_name AS CHAR), CAST(data_type AS CHAR), CAST(is_nullable AS CHAR)
    FROM information_schema.columns
    WHERE table_schema = ? AND table_name = ? ORDER BY ordinal_position";

pub async fn list_columns(pool: &DbPool, namespace: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let raw: Vec<(String, String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(COLUMNS_SQL_PG).bind(namespace).bind(table).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(COLUMNS_SQL_MYSQL).bind(namespace).bind(table).fetch_all(p).await?,
    };
    let pk: Vec<String> = primary_key_with_types(pool, namespace, table)
        .await?
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    Ok(raw
        .into_iter()
        .map(|(name, data_type, is_nullable)| ColumnInfo {
            is_primary_key: pk.contains(&name),
            nullable: is_nullable.eq_ignore_ascii_case("YES"),
            name,
            data_type,
        })
        .collect())
}

const INT_TYPES: &[&str] = &["smallint", "integer", "bigint", "int", "tinyint", "mediumint"];

/// 仅当表有单列整型主键时返回列名，用于 keyset 分页。
pub async fn integer_primary_key(pool: &DbPool, namespace: &str, table: &str) -> Result<Option<String>, AppError> {
    let pk = primary_key_with_types(pool, namespace, table).await?;
    match pk.as_slice() {
        [(name, data_type)] if INT_TYPES.contains(&data_type.to_lowercase().as_str()) => Ok(Some(name.clone())),
        _ => Ok(None),
    }
}
