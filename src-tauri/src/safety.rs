use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

pub fn analyze(sql: &str) -> Vec<DangerWarning> {
    let cleaned = strip_literals_and_comments(sql);
    let mut warnings = Vec::new();
    for raw_stmt in cleaned.split(';') {
        let stmt = raw_stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        let upper = stmt.to_uppercase();
        let has_where = upper.split_whitespace().any(|w| w == "WHERE");
        let (first, second) = first_two_words(stmt);
        let kind = match (first.as_str(), second.as_str()) {
            ("DROP", "DATABASE") | ("DROP", "SCHEMA") => Some(DangerKind::DropDatabase),
            ("DROP", "TABLE") => Some(DangerKind::DropTable),
            ("TRUNCATE", _) => Some(DangerKind::Truncate),
            ("DELETE", _) if !has_where => Some(DangerKind::DeleteWithoutWhere),
            ("UPDATE", _) if !has_where => Some(DangerKind::UpdateWithoutWhere),
            _ => None,
        };
        if let Some(kind) = kind {
            warnings.push(DangerWarning { kind, statement: stmt.to_string() });
        }
    }
    warnings
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
}
