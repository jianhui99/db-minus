use crate::connection::pool::DbPool;
use crate::error::AppError;
use crate::query;
use crate::safety;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedStatement {
    /// 1-based ordinal position among the statements found in the script
    /// (human-facing, not a 0-based array index).
    pub index: usize,
    pub sql: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub total_statements: usize,
    pub executed_statements: usize,
    pub duration_ms: u64,
    pub failed_statement: Option<FailedStatement>,
}

/// Splits `script` into statements (via `safety::split_statements`) and
/// executes them sequentially against `pool`, one at a time, with NO
/// wrapping transaction. Stops at the first failing statement; whatever
/// executed before it remains committed. Reuses `query::execute_sql` per
/// statement so row/DDL classification and the per-statement 30s timeout
/// are inherited unchanged.
pub async fn run_import(pool: &DbPool, script: &str) -> Result<ImportResult, AppError> {
    let statements = safety::split_statements(script);
    let started = Instant::now();
    let mut executed = 0usize;
    let mut failed_statement = None;

    for (i, stmt) in statements.iter().enumerate() {
        match query::execute_sql(pool, stmt).await {
            Ok(_) => executed += 1,
            Err(e) => {
                failed_statement = Some(FailedStatement {
                    index: i + 1,
                    sql: stmt.clone(),
                    message: e.to_string(),
                });
                break;
            }
        }
    }

    Ok(ImportResult {
        total_statements: statements.len(),
        executed_statements: executed,
        duration_ms: started.elapsed().as_millis() as u64,
        failed_statement,
    })
}
