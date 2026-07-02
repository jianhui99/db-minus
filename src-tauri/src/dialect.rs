use crate::connection::config::{ConnectionConfig, Driver, SslMode};

pub fn quote_ident(driver: Driver, ident: &str) -> String {
    match driver {
        Driver::Postgres => format!("\"{}\"", ident.replace('"', "\"\"")),
        Driver::MySql => format!("`{}`", ident.replace('`', "``")),
    }
}

pub fn qualified_table(driver: Driver, namespace: &str, table: &str) -> String {
    format!("{}.{}", quote_ident(driver, namespace), quote_ident(driver, table))
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn connect_url(config: &ConnectionConfig, password: &str) -> String {
    let scheme = match config.driver {
        Driver::Postgres => "postgres",
        Driver::MySql => "mysql",
    };
    let ssl = match (config.driver, config.ssl_mode) {
        (Driver::Postgres, SslMode::Disable) => "sslmode=disable",
        (Driver::Postgres, SslMode::Prefer) => "sslmode=prefer",
        (Driver::Postgres, SslMode::Require) => "sslmode=require",
        (Driver::MySql, SslMode::Disable) => "ssl-mode=DISABLED",
        (Driver::MySql, SslMode::Prefer) => "ssl-mode=PREFERRED",
        (Driver::MySql, SslMode::Require) => "ssl-mode=REQUIRED",
    };
    format!(
        "{}://{}:{}@{}:{}/{}?{}",
        scheme,
        url_encode(&config.username),
        url_encode(password),
        config.host,
        config.port,
        url_encode(&config.database),
        ssl
    )
}

pub fn placeholder(driver: Driver, n: usize) -> String {
    match driver {
        Driver::Postgres => format!("${}", n),
        Driver::MySql => {
            let _ = n;
            "?".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::config::{ConnectionConfig, Driver, SslMode};

    fn cfg(driver: Driver, ssl: SslMode) -> ConnectionConfig {
        ConnectionConfig {
            id: "x".into(),
            name: "x".into(),
            driver,
            host: "localhost".into(),
            port: 5433,
            username: "dbminus".into(),
            database: "dbminus_test".into(),
            ssl_mode: ssl,
        }
    }

    #[test]
    fn quotes_postgres_idents() {
        assert_eq!(quote_ident(Driver::Postgres, "users"), "\"users\"");
        assert_eq!(quote_ident(Driver::Postgres, "we\"ird"), "\"we\"\"ird\"");
    }

    #[test]
    fn quotes_mysql_idents() {
        assert_eq!(quote_ident(Driver::MySql, "users"), "`users`");
        assert_eq!(quote_ident(Driver::MySql, "we`ird"), "`we``ird`");
    }

    #[test]
    fn qualifies_table() {
        assert_eq!(qualified_table(Driver::Postgres, "public", "users"), "\"public\".\"users\"");
        assert_eq!(qualified_table(Driver::MySql, "dbminus_test", "users"), "`dbminus_test`.`users`");
    }

    #[test]
    fn builds_postgres_url() {
        let url = connect_url(&cfg(Driver::Postgres, SslMode::Require), "p@ss");
        assert_eq!(url, "postgres://dbminus:p%40ss@localhost:5433/dbminus_test?sslmode=require");
    }

    #[test]
    fn builds_mysql_url() {
        let url = connect_url(&cfg(Driver::MySql, SslMode::Disable), "pass");
        assert_eq!(url, "mysql://dbminus:pass@localhost:5433/dbminus_test?ssl-mode=DISABLED");
    }

    #[test]
    fn placeholders() {
        assert_eq!(placeholder(Driver::Postgres, 1), "$1");
        assert_eq!(placeholder(Driver::MySql, 3), "?");
    }
}
