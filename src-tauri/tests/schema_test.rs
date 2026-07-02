mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::connect;
use db_minus_lib::schema::{integer_primary_key, list_columns, list_namespaces, list_tables, TableKind};

#[tokio::test]
async fn pg_namespaces_include_public() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let ns = list_namespaces(&pool).await.unwrap();
    assert!(ns.contains(&"public".to_string()));
    assert!(!ns.iter().any(|n| n.starts_with("pg_")));
}

#[tokio::test]
async fn mysql_namespaces_include_test_db() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    let ns = list_namespaces(&pool).await.unwrap();
    assert!(ns.contains(&"dbminus_test".to_string()));
    assert!(!ns.contains(&"mysql".to_string()));
}

#[tokio::test]
async fn pg_tables_and_views() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let tables = list_tables(&pool, "public").await.unwrap();
    let users = tables.iter().find(|t| t.name == "users").unwrap();
    assert_eq!(users.kind, TableKind::Table);
    let view = tables.iter().find(|t| t.name == "active_users").unwrap();
    assert_eq!(view.kind, TableKind::View);
}

#[tokio::test]
async fn pg_columns_mark_primary_key() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let cols = list_columns(&pool, "public", "users").await.unwrap();
    let id = cols.iter().find(|c| c.name == "id").unwrap();
    assert!(id.is_primary_key);
    assert!(!id.nullable);
    let name = cols.iter().find(|c| c.name == "full_name").unwrap();
    assert!(name.nullable);
    assert!(!name.is_primary_key);
}

#[tokio::test]
async fn integer_pk_detection() {
    let pg = connect(&pg_config(), PASSWORD).await.unwrap();
    assert_eq!(integer_primary_key(&pg, "public", "users").await.unwrap(), Some("id".to_string()));
    assert_eq!(integer_primary_key(&pg, "public", "app_log").await.unwrap(), None);
    assert_eq!(integer_primary_key(&pg, "public", "types_demo").await.unwrap(), None);

    let my = connect(&mysql_config(), PASSWORD).await.unwrap();
    assert_eq!(integer_primary_key(&my, "dbminus_test", "users").await.unwrap(), Some("id".to_string()));
}
