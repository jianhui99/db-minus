mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::connect;
use db_minus_lib::query::execute_sql;
use serde_json::Value;

#[tokio::test]
async fn pg_select_returns_rows_and_columns() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT id, email, is_active, balance FROM users ORDER BY id LIMIT 3").await.unwrap();
    assert_eq!(r.columns.len(), 4);
    assert_eq!(r.columns[1].name, "email");
    assert_eq!(r.rows.len(), 3);
    assert_eq!(r.rows[0][0], Value::from(1));
    assert_eq!(r.rows[0][1], Value::from("user1@example.com"));
    assert_eq!(r.rows[0][2], Value::from(true));
    assert!(r.rows[0][3].is_string(), "NUMERIC should serialize as string, got {:?}", r.rows[0][3]);
    assert!(!r.truncated);
    assert_eq!(r.affected_rows, None);
}

#[tokio::test]
async fn mysql_select_returns_rows() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT id, email, is_active FROM users ORDER BY id LIMIT 2").await.unwrap();
    assert_eq!(r.rows.len(), 2);
    assert_eq!(r.rows[0][0], Value::from(1));
    assert_eq!(r.rows[0][1], Value::from("user1@example.com"));
}

#[tokio::test]
async fn mysql_timestamp_column_decodes_to_rfc3339() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT created_at FROM users ORDER BY id LIMIT 1").await.unwrap();
    let value = r.rows[0][0].as_str().expect("TIMESTAMP should decode to a string, not fall back to <timestamp>");
    assert!(value.contains('T'), "expected RFC3339-ish format, got {value}");
    assert!(value.ends_with("+00:00"), "expected UTC offset suffix, got {value}");
}

#[tokio::test]
async fn pg_types_demo_serializes() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT id, payload, raw, ratio, born, wake FROM types_demo ORDER BY payload NULLS LAST").await.unwrap();
    assert_eq!(r.rows.len(), 2);
    let first = &r.rows[0];
    assert!(first[0].is_string(), "uuid should be string, got {:?}", first[0]);
    assert!(first[1].is_object(), "jsonb should be object, got {:?}", first[1]);
    assert_eq!(first[2], Value::from("0xdeadbeef"));
    assert_eq!(first[3], Value::from(3.14));
    assert_eq!(first[4], Value::from("1990-05-04"));
    let second = &r.rows[1];
    assert_eq!(second[1], Value::Null);
}

#[tokio::test]
async fn dml_reports_affected_rows() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "CREATE TABLE IF NOT EXISTS scratch (n INT)").await.unwrap();
    let r = execute_sql(&pool, "INSERT INTO scratch VALUES (1), (2), (3)").await.unwrap();
    assert_eq!(r.affected_rows, Some(3));
    assert!(r.columns.is_empty());
    execute_sql(&pool, "DROP TABLE scratch").await.unwrap();
}

#[tokio::test]
async fn duration_is_recorded() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT pg_sleep(0.05)").await.unwrap();
    assert!(r.duration_ms >= 50, "duration was {}", r.duration_ms);
}

#[tokio::test]
async fn sql_error_is_query_error() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let err = execute_sql(&pool, "SELECT * FROM does_not_exist").await.unwrap_err();
    assert!(format!("{err}").contains("query failed"));
}
