#![allow(dead_code)]
use db_minus_lib::connection::config::{ConnectionConfig, Driver, SslMode};

pub fn pg_config() -> ConnectionConfig {
    ConnectionConfig {
        id: "test-pg".into(),
        name: "test pg".into(),
        driver: Driver::Postgres,
        host: "localhost".into(),
        port: std::env::var("DB_MINUS_TEST_PG_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5433),
        username: "dbminus".into(),
        database: "dbminus_test".into(),
        ssl_mode: SslMode::Disable,
    }
}

pub fn mysql_config() -> ConnectionConfig {
    ConnectionConfig {
        id: "test-mysql".into(),
        name: "test mysql".into(),
        driver: Driver::MySql,
        host: "localhost".into(),
        port: std::env::var("DB_MINUS_TEST_MYSQL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3307),
        username: "dbminus".into(),
        database: "dbminus_test".into(),
        ssl_mode: SslMode::Disable,
    }
}

pub const PASSWORD: &str = "dbminus";
