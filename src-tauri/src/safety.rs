use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DangerKind {
    DropDatabase,
    DropTable,
    Truncate,
    DeleteWithoutWhere,
    UpdateWithoutWhere,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DangerWarning {
    pub kind: DangerKind,
    pub statement: String,
}

/// 把字符串字面量与注释抹成空格，保留其余字符结构，避免误报。
fn strip_literals_and_comments(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut out = String::with_capacity(sql.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                out.push(' ');
                i += 1;
                while i < bytes.len() {
                    let cc = bytes[i] as char;
                    i += 1;
                    if cc == quote {
                        // 连续两个引号是转义，继续扫描
                        if i < bytes.len() && bytes[i] as char == quote {
                            i += 1;
                            continue;
                        }
                        break;
                    }
                }
            }
            '-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                out.push(' ');
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn first_two_words(stmt: &str) -> (String, String) {
    let mut words = stmt.split_whitespace();
    let a = words.next().unwrap_or("").to_uppercase();
    let b = words.next().unwrap_or("").to_uppercase();
    (a, b)
}

/// 按顶层（不在字符串/标识符字面量或注释内）的分号切分 `sql`，返回原始子串
/// （trim 后，空语句丢弃）。与 `strip_literals_and_comments` 共用同一套引号/注释
/// 扫描规则，但不抹除内容，只用它定位语句边界，供执行场景直接复用原文。
///
/// 已知局限（沿用本模块既有限制，不在此修复）：不识别 PL/pgSQL 的 `$$...$$`
/// 美元引用块，块内的分号会被当作语句边界误切。
pub(crate) fn split_statements(sql: &str) -> Vec<String> {
    let bytes = sql.as_bytes();
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                i += 1;
                while i < bytes.len() {
                    let cc = bytes[i] as char;
                    i += 1;
                    if cc == quote {
                        if i < bytes.len() && bytes[i] as char == quote {
                            i += 1;
                            continue;
                        }
                        break;
                    }
                }
            }
            '-' if i + 1 < bytes.len() && bytes[i + 1] == b'-' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            ';' => {
                let stmt = sql[start..i].trim();
                if !stmt.is_empty() {
                    statements.push(stmt.to_string());
                }
                i += 1;
                start = i;
            }
            _ => {
                i += 1;
            }
        }
    }
    let tail = sql[start..].trim();
    if !tail.is_empty() {
        statements.push(tail.to_string());
    }
    statements
}

fn classify(stmt: &str) -> Option<DangerKind> {
    let cleaned = strip_literals_and_comments(stmt);
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }
    let upper = trimmed.to_uppercase();
    let has_where = upper.split_whitespace().any(|w| w == "WHERE");
    let (first, second) = first_two_words(trimmed);
    match (first.as_str(), second.as_str()) {
        ("DROP", "DATABASE") | ("DROP", "SCHEMA") => Some(DangerKind::DropDatabase),
        ("DROP", "TABLE") => Some(DangerKind::DropTable),
        ("TRUNCATE", _) => Some(DangerKind::Truncate),
        ("DELETE", _) if !has_where => Some(DangerKind::DeleteWithoutWhere),
        ("UPDATE", _) if !has_where => Some(DangerKind::UpdateWithoutWhere),
        _ => None,
    }
}

pub fn analyze(sql: &str) -> Vec<DangerWarning> {
    split_statements(sql)
        .into_iter()
        .filter_map(|stmt| classify(&stmt).map(|kind| DangerWarning { kind, statement: stmt }))
        .collect()
}

pub fn danger_label(kind: DangerKind) -> &'static str {
    match kind {
        DangerKind::DropDatabase => "DROP DATABASE",
        DangerKind::DropTable => "DROP TABLE",
        DangerKind::Truncate => "TRUNCATE",
        DangerKind::DeleteWithoutWhere => "DELETE without WHERE",
        DangerKind::UpdateWithoutWhere => "UPDATE without WHERE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(sql: &str) -> Vec<DangerKind> {
        analyze(sql).into_iter().map(|w| w.kind).collect()
    }

    #[test]
    fn safe_statements_pass() {
        assert!(kinds("SELECT * FROM users").is_empty());
        assert!(kinds("DELETE FROM users WHERE id = 1").is_empty());
        assert!(kinds("UPDATE users SET a = 1 WHERE id = 1").is_empty());
        assert!(kinds("INSERT INTO t VALUES (1)").is_empty());
    }

    #[test]
    fn detects_drop_and_truncate() {
        assert_eq!(kinds("DROP TABLE users"), vec![DangerKind::DropTable]);
        assert_eq!(kinds("drop database prod"), vec![DangerKind::DropDatabase]);
        assert_eq!(kinds("TRUNCATE TABLE users"), vec![DangerKind::Truncate]);
    }

    #[test]
    fn detects_missing_where() {
        assert_eq!(kinds("DELETE FROM users"), vec![DangerKind::DeleteWithoutWhere]);
        assert_eq!(kinds("UPDATE users SET active = false"), vec![DangerKind::UpdateWithoutWhere]);
    }

    #[test]
    fn ignores_keywords_inside_strings_and_comments() {
        assert!(kinds("SELECT 'DROP TABLE x' FROM t").is_empty());
        assert!(kinds("SELECT 1 -- DROP TABLE x\nFROM t").is_empty());
        assert!(kinds("SELECT 1 /* TRUNCATE y */ FROM t").is_empty());
    }

    #[test]
    fn where_inside_string_does_not_count() {
        assert_eq!(kinds("DELETE FROM users -- where id = 1"), vec![DangerKind::DeleteWithoutWhere]);
    }

    #[test]
    fn multiple_statements_report_each() {
        let ks = kinds("DELETE FROM a; DROP TABLE b;");
        assert_eq!(ks, vec![DangerKind::DeleteWithoutWhere, DangerKind::DropTable]);
    }

    #[test]
    fn analyze_statement_text_is_unmodified_original() {
        let warnings = analyze("DELETE FROM users -- trailing comment");
        assert_eq!(warnings[0].statement, "DELETE FROM users -- trailing comment");
    }

    #[test]
    fn split_basic_statements() {
        assert_eq!(split_statements("SELECT 1; SELECT 2;"), vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_preserves_semicolons_inside_string_literals() {
        let stmts = split_statements("INSERT INTO t VALUES ('a;b'); SELECT 1;");
        assert_eq!(stmts, vec!["INSERT INTO t VALUES ('a;b')", "SELECT 1"]);
    }

    #[test]
    fn split_ignores_semicolons_in_line_comments() {
        let stmts = split_statements("SELECT 1; -- drop table x;\nSELECT 2;");
        assert_eq!(stmts, vec!["SELECT 1", "-- drop table x;\nSELECT 2"]);
    }

    #[test]
    fn split_ignores_semicolons_in_block_comments() {
        let stmts = split_statements("SELECT 1; /* a; b */ SELECT 2;");
        assert_eq!(stmts, vec!["SELECT 1", "/* a; b */ SELECT 2"]);
    }

    #[test]
    fn split_drops_empty_statements() {
        assert_eq!(split_statements("SELECT 1;;  ; SELECT 2"), vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_handles_no_trailing_semicolon() {
        assert_eq!(split_statements("SELECT 1; SELECT 2"), vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn split_empty_input_yields_no_statements() {
        assert!(split_statements("   \n  ").is_empty());
    }
}
