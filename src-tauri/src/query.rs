use crate::connection::pool::DbPool;
use crate::error::AppError;
use futures::TryStreamExt;
use serde::Serialize;
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::time::{Duration, Instant};

pub const MAX_RESULT_ROWS: usize = 10_000;
const QUERY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMeta {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub truncated: bool,
}

fn hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// 依次尝试解码器，命中即返回；宏内用 return，调用方必须是独立函数。
macro_rules! try_decode {
    ($row:expr, $i:expr, $($t:ty => $conv:expr),+ $(,)?) => {{
        $(
            if let Ok(v) = $row.try_get::<$t, _>($i) {
                #[allow(clippy::redundant_closure_call)]
                { return ($conv)(v); }
            }
        )+
    }};
}

fn pg_value(row: &PgRow, i: usize) -> Value {
    let Ok(raw) = row.try_get_raw(i) else { return Value::Null };
    if raw.is_null() {
        return Value::Null;
    }
    let type_name = raw.type_info().name().to_string();
    pg_value_typed(row, i, &type_name)
}

fn pg_value_typed(row: &PgRow, i: usize, type_name: &str) -> Value {
    match type_name {
        "BOOL" => try_decode!(row, i, bool => Value::from),
        "INT2" => try_decode!(row, i, i16 => Value::from),
        "INT4" => try_decode!(row, i, i32 => Value::from),
        "INT8" => try_decode!(row, i, i64 => Value::from),
        "FLOAT4" => try_decode!(row, i, f32 => |v: f32| Value::from(v as f64)),
        "FLOAT8" => try_decode!(row, i, f64 => Value::from),
        "NUMERIC" => try_decode!(row, i, rust_decimal::Decimal => |v: rust_decimal::Decimal| Value::from(v.to_string())),
        "UUID" => try_decode!(row, i, uuid::Uuid => |v: uuid::Uuid| Value::from(v.to_string())),
        "JSON" | "JSONB" => try_decode!(row, i, Value => |v| v),
        "TIMESTAMPTZ" => try_decode!(row, i, chrono::DateTime<chrono::Utc> => |v: chrono::DateTime<chrono::Utc>| Value::from(v.to_rfc3339())),
        "TIMESTAMP" => try_decode!(row, i, chrono::NaiveDateTime => |v: chrono::NaiveDateTime| Value::from(v.to_string())),
        "DATE" => try_decode!(row, i, chrono::NaiveDate => |v: chrono::NaiveDate| Value::from(v.to_string())),
        "TIME" => try_decode!(row, i, chrono::NaiveTime => |v: chrono::NaiveTime| Value::from(v.to_string())),
        "BYTEA" => try_decode!(row, i, Vec<u8> => |v: Vec<u8>| Value::from(hex_string(&v))),
        _ => {}
    }
    try_decode!(row, i, String => Value::from);
    Value::from(format!("<{}>", type_name.to_lowercase()))
}

fn mysql_value(row: &MySqlRow, i: usize) -> Value {
    let Ok(raw) = row.try_get_raw(i) else { return Value::Null };
    if raw.is_null() {
        return Value::Null;
    }
    let type_name = raw.type_info().name().to_string();
    mysql_value_typed(row, i, &type_name)
}

fn mysql_value_typed(row: &MySqlRow, i: usize, type_name: &str) -> Value {
    let unsigned = type_name.contains("UNSIGNED");
    match type_name {
        "BOOLEAN" => try_decode!(row, i, bool => Value::from, i8 => |v: i8| Value::from(v != 0)),
        t if t.starts_with("TINYINT")
            || t.starts_with("SMALLINT")
            || t.starts_with("MEDIUMINT")
            || t.starts_with("INT")
            || t.starts_with("BIGINT")
            || t == "YEAR" =>
        {
            if unsigned {
                try_decode!(row, i, u64 => Value::from, i64 => Value::from);
            } else {
                try_decode!(row, i, i64 => Value::from, i32 => Value::from);
            }
        }
        "FLOAT" => try_decode!(row, i, f32 => |v: f32| Value::from(v as f64)),
        "DOUBLE" => try_decode!(row, i, f64 => Value::from),
        "DECIMAL" => try_decode!(row, i, rust_decimal::Decimal => |v: rust_decimal::Decimal| Value::from(v.to_string())),
        "JSON" => try_decode!(row, i, Value => |v| v),
        "DATETIME" | "TIMESTAMP" => try_decode!(row, i, chrono::NaiveDateTime => |v: chrono::NaiveDateTime| Value::from(v.to_string())),
        "DATE" => try_decode!(row, i, chrono::NaiveDate => |v: chrono::NaiveDate| Value::from(v.to_string())),
        "TIME" => try_decode!(row, i, chrono::NaiveTime => |v: chrono::NaiveTime| Value::from(v.to_string())),
        t if t.contains("BLOB") || t.contains("BINARY") => try_decode!(row, i, Vec<u8> => |v: Vec<u8>| Value::from(hex_string(&v))),
        _ => {}
    }
    try_decode!(row, i, String => Value::from);
    Value::from(format!("<{}>", type_name.to_lowercase()))
}

pub(crate) fn pg_row_to_values(row: &PgRow) -> Vec<Value> {
    (0..row.columns().len()).map(|i| pg_value(row, i)).collect()
}

pub(crate) fn mysql_row_to_values(row: &MySqlRow) -> Vec<Value> {
    (0..row.columns().len()).map(|i| mysql_value(row, i)).collect()
}

pub(crate) fn pg_columns(row: &PgRow) -> Vec<ColumnMeta> {
    row.columns()
        .iter()
        .map(|c| ColumnMeta { name: c.name().to_string(), type_name: c.type_info().name().to_lowercase() })
        .collect()
}

pub(crate) fn mysql_columns(row: &MySqlRow) -> Vec<ColumnMeta> {
    row.columns()
        .iter()
        .map(|c| ColumnMeta { name: c.name().to_string(), type_name: c.type_info().name().to_lowercase() })
        .collect()
}

fn first_keyword(sql: &str) -> String {
    sql.split_whitespace().next().unwrap_or("").to_uppercase()
}

fn is_row_returning(sql: &str) -> bool {
    matches!(
        first_keyword(sql).as_str(),
        "SELECT" | "WITH" | "SHOW" | "EXPLAIN" | "DESCRIBE" | "DESC" | "TABLE" | "VALUES"
    )
}

pub async fn execute_sql(pool: &DbPool, sql: &str) -> Result<QueryResult, AppError> {
    let started = Instant::now();
    let work = async {
        if is_row_returning(sql) {
            fetch_rows(pool, sql, MAX_RESULT_ROWS).await
        } else {
            let affected = match pool {
                DbPool::Postgres(p) => sqlx::query(sql).execute(p).await?.rows_affected(),
                DbPool::MySql(p) => sqlx::query(sql).execute(p).await?.rows_affected(),
            };
            Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: Some(affected),
                duration_ms: 0,
                truncated: false,
            })
        }
    };
    let mut result = tokio::time::timeout(QUERY_TIMEOUT, work)
        .await
        .map_err(|_| AppError::Timeout("query timed out after 30s".into()))??;
    result.duration_ms = started.elapsed().as_millis() as u64;
    Ok(result)
}

async fn fetch_rows(pool: &DbPool, sql: &str, max_rows: usize) -> Result<QueryResult, AppError> {
    let mut columns: Vec<ColumnMeta> = vec![];
    let mut rows: Vec<Vec<Value>> = vec![];
    let mut truncated = false;

    match pool {
        DbPool::Postgres(p) => {
            let mut stream = sqlx::query(sql).fetch(p);
            while let Some(row) = stream.try_next().await? {
                if columns.is_empty() {
                    columns = pg_columns(&row);
                }
                if rows.len() >= max_rows {
                    truncated = true;
                    break;
                }
                rows.push(pg_row_to_values(&row));
            }
        }
        DbPool::MySql(p) => {
            let mut stream = sqlx::query(sql).fetch(p);
            while let Some(row) = stream.try_next().await? {
                if columns.is_empty() {
                    columns = mysql_columns(&row);
                }
                if rows.len() >= max_rows {
                    truncated = true;
                    break;
                }
                rows.push(mysql_row_to_values(&row));
            }
        }
    }

    Ok(QueryResult { columns, rows, affected_rows: None, duration_ms: 0, truncated })
}
