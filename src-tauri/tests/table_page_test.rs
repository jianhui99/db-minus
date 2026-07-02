mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::connect;
use db_minus_lib::query::{fetch_table_page, Cursor, Sort, TablePageRequest};
use serde_json::Value;

fn req(namespace: &str, table: &str) -> TablePageRequest {
    TablePageRequest {
        namespace: namespace.into(),
        table: table.into(),
        sort: None,
        cursor: None,
        limit: 500,
    }
}

#[tokio::test]
async fn pg_keyset_pagination_walks_all_rows() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let mut request = req("public", "users");
    let mut total = 0;
    let mut last_id = 0i64;
    loop {
        let page = fetch_table_page(&pool, &request).await.unwrap();
        total += page.rows.len();
        if let Some(first) = page.rows.first() {
            let id = first[0].as_i64().unwrap();
            assert!(id > last_id, "keyset must be strictly increasing");
            last_id = page.rows.last().unwrap()[0].as_i64().unwrap();
        }
        match page.next_cursor {
            Some(c) => {
                assert!(matches!(c, Cursor::Keyset { .. }));
                request.cursor = Some(c);
            }
            None => break,
        }
    }
    assert_eq!(total, 1500);
}

#[tokio::test]
async fn mysql_keyset_pagination_walks_all_rows() {
    let pool = connect(&mysql_config(), PASSWORD).await.unwrap();
    let mut request = req("dbminus_test", "users");
    let mut total = 0;
    loop {
        let page = fetch_table_page(&pool, &request).await.unwrap();
        total += page.rows.len();
        match page.next_cursor {
            Some(c) => request.cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(total, 1500);
}

#[tokio::test]
async fn table_without_pk_uses_offset() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let mut request = req("public", "app_log");
    request.limit = 25;
    let page = fetch_table_page(&pool, &request).await.unwrap();
    assert_eq!(page.rows.len(), 25);
    let next = page.next_cursor.unwrap();
    assert!(matches!(next, Cursor::Offset { offset: 25 }));
    request.cursor = Some(next);
    let page2 = fetch_table_page(&pool, &request).await.unwrap();
    assert_eq!(page2.rows.len(), 15);
    assert!(page2.next_cursor.is_none());
}

#[tokio::test]
async fn custom_sort_uses_offset_and_orders() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let mut request = req("public", "users");
    request.sort = Some(Sort { column: "id".into(), desc: true });
    request.limit = 10;
    let page = fetch_table_page(&pool, &request).await.unwrap();
    assert_eq!(page.rows[0][0], Value::from(1500));
    assert!(matches!(page.next_cursor, Some(Cursor::Offset { offset: 10 })));
}

#[tokio::test]
async fn sort_column_is_validated() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let mut request = req("public", "users");
    request.sort = Some(Sort { column: "nope; DROP TABLE users".into(), desc: false });
    let err = fetch_table_page(&pool, &request).await.unwrap_err();
    assert!(format!("{err}").contains("not found"));
}
