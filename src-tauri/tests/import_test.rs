mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::connect;
use db_minus_lib::import::run_import;
use db_minus_lib::query::execute_sql;
use serde_json::Value;

#[tokio::test]
async fn pg_full_script_executes_successfully() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "DROP TABLE IF EXISTS import_full").await.unwrap();

    let script = "CREATE TABLE import_full (n INT); \
                   INSERT INTO import_full VALUES (1), (2); \
                   INSERT INTO import_full VALUES (3);";
    let result = run_import(&pool, script).await.unwrap();
    assert_eq!(result.total_statements, 3);
    assert_eq!(result.executed_statements, 3);
    assert!(result.failed_statement.is_none());

    let r = execute_sql(&pool, "SELECT COUNT(*) FROM import_full").await.unwrap();
    assert_eq!(r.rows[0][0], Value::from(3));

    execute_sql(&pool, "DROP TABLE import_full").await.unwrap();
}

#[tokio::test]
async fn mysql_full_script_executes_successfully() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "DROP TABLE IF EXISTS import_full").await.unwrap();

    let script = "CREATE TABLE import_full (n INT); \
                   INSERT INTO import_full VALUES (1), (2); \
                   INSERT INTO import_full VALUES (3);";
    let result = run_import(&pool, script).await.unwrap();
    assert_eq!(result.total_statements, 3);
    assert_eq!(result.executed_statements, 3);
    assert!(result.failed_statement.is_none());

    let r = execute_sql(&pool, "SELECT COUNT(*) FROM import_full").await.unwrap();
    assert_eq!(r.rows[0][0], Value::from(3));

    execute_sql(&pool, "DROP TABLE import_full").await.unwrap();
}

#[tokio::test]
async fn pg_stops_at_first_failure_and_leaves_prior_statements_committed() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "DROP TABLE IF EXISTS import_partial").await.unwrap();

    let script = "CREATE TABLE import_partial (n INT); \
                   INSERT INTO import_partial VALUES (1); \
                   INSERT INTO does_not_exist VALUES (1); \
                   INSERT INTO import_partial VALUES (2);";
    let result = run_import(&pool, script).await.unwrap();
    assert_eq!(result.total_statements, 4);
    assert_eq!(result.executed_statements, 2);
    let failed = result.failed_statement.expect("expected a failed statement");
    assert_eq!(failed.index, 3);
    assert!(failed.message.contains("does_not_exist"), "message was: {}", failed.message);

    let r = execute_sql(&pool, "SELECT n FROM import_partial ORDER BY n").await.unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0][0], Value::from(1));

    execute_sql(&pool, "DROP TABLE import_partial").await.unwrap();
}

#[tokio::test]
async fn mysql_stops_at_first_failure_and_leaves_prior_statements_committed() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "DROP TABLE IF EXISTS import_partial").await.unwrap();

    let script = "CREATE TABLE import_partial (n INT); \
                   INSERT INTO import_partial VALUES (1); \
                   INSERT INTO does_not_exist VALUES (1); \
                   INSERT INTO import_partial VALUES (2);";
    let result = run_import(&pool, script).await.unwrap();
    assert_eq!(result.total_statements, 4);
    assert_eq!(result.executed_statements, 2);
    let failed = result.failed_statement.expect("expected a failed statement");
    assert_eq!(failed.index, 3);
    assert!(failed.message.contains("does_not_exist"), "message was: {}", failed.message);

    let r = execute_sql(&pool, "SELECT n FROM import_partial ORDER BY n").await.unwrap();
    assert_eq!(r.rows.len(), 1);
    assert_eq!(r.rows[0][0], Value::from(1));

    execute_sql(&pool, "DROP TABLE import_partial").await.unwrap();
}

#[tokio::test]
async fn semicolon_inside_string_literal_does_not_break_statements() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    execute_sql(&pool, "DROP TABLE IF EXISTS import_literal").await.unwrap();

    let script = "CREATE TABLE import_literal (label TEXT); \
                   INSERT INTO import_literal VALUES ('a;b');";
    let result = run_import(&pool, script).await.unwrap();
    assert_eq!(result.total_statements, 2);
    assert_eq!(result.executed_statements, 2);
    assert!(result.failed_statement.is_none());

    let r = execute_sql(&pool, "SELECT label FROM import_literal").await.unwrap();
    assert_eq!(r.rows[0][0], Value::from("a;b"));

    execute_sql(&pool, "DROP TABLE import_literal").await.unwrap();
}

#[tokio::test]
async fn empty_or_whitespace_script_reports_zero_statements() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let result = run_import(&pool, "   \n  ").await.unwrap();
    assert_eq!(result.total_statements, 0);
    assert_eq!(result.executed_statements, 0);
    assert!(result.failed_statement.is_none());
}
