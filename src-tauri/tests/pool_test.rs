mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::{test_connection, DbPool, PoolManager};

#[tokio::test]
async fn test_connection_ok_postgres() {
    test_connection(&pg_config(), PASSWORD).await.unwrap();
}

#[tokio::test]
async fn test_connection_ok_mysql() {
    test_connection(&mysql_config(), PASSWORD).await.unwrap();
}

#[tokio::test]
async fn test_connection_bad_password_fails() {
    let err = test_connection(&pg_config(), "wrong").await.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("connection failed") || msg.contains("timeout"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn pool_manager_caches_pool() {
    let mgr = PoolManager::new();
    let cfg = pg_config();
    let a = mgr.get_or_create(&cfg, PASSWORD).await.unwrap();
    let b = mgr.get_or_create(&cfg, PASSWORD).await.unwrap();
    match (a, b) {
        (DbPool::Postgres(pa), DbPool::Postgres(pb)) => {
            assert_eq!(pa.size(), pb.size());
        }
        _ => panic!("expected postgres pools"),
    }
    mgr.remove("test-pg").await;
}
