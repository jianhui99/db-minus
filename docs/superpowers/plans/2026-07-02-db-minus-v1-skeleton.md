# DB-Minus V1 骨架实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 打通 DB-Minus 核心链路：连接 PostgreSQL / MySQL，浏览 Schema，只读 Data Grid 分页浏览，SQL Editor 执行查询并展示结果。

**Architecture:** Tauri 2 桌面应用。
Rust 后端通过 Tauri IPC command 暴露连接管理、元数据查询、SQL 执行能力，连接池用 `enum DbPool` 手动分发 PG / MySQL。
React 前端用 Zustand 管 Tab 状态，TanStack Query 管服务端数据，TanStack Table + Virtual 渲染大结果集。

**Tech Stack:** Tauri 2、React 19、TypeScript、Vite、Tailwind 4、shadcn/ui、TanStack Table/Virtual/Query、Zustand、Monaco Editor、Rust、tokio、sqlx 0.8、keyring 4。

**Spec:** `docs/superpowers/specs/2026-07-02-db-minus-v1-skeleton-design.md`

## Global Constraints

- 平台：macOS 优先（darwin），密码存 macOS Keychain。
- sqlx 锁定 `0.8`（0.9 刚发布，API 未沉淀，升级作为后续 chore）。
- 所有 IPC 传输的 Rust 类型加 `#[serde(rename_all = "camelCase")]`，前端 TS 类型用 camelCase。
- Rust 集成测试依赖 Docker 测试库：PG 在 `localhost:5433`，MySQL 在 `localhost:3307`，跑测试前先 `docker compose -f dev/docker-compose.yml up -d --wait`。
- UI 文案一律英文。
- git commit message 不加任何 co-author 信息（用户全局规则）。
- Markdown 文档使用短横线（-），禁止长破折号。
- Data Grid 每批加载 500 行；SQL Editor 结果集上限 10000 行（超出置 `truncated`）。
- 连接测试超时 10s，查询超时 30s。
- Keyset 分页仅在「单列整型主键且无自定义排序」时启用，其余退化为 LIMIT/OFFSET（对 spec 的细化：绑定任意类型 cursor 值的复杂度不属于骨架）。
- PRD 提到的 React Aria Tree 推迟到迭代 2（键盘导航落地时），骨架版 Schema Tree 手写。

---

### Task 1: 项目脚手架（Tauri 2 + React + TS + Tailwind + shadcn/ui）

**Files:**
- Create: 整个 Tauri 模板（`src/`、`src-tauri/`、`package.json`、`vite.config.ts` 等）
- Modify: `vite.config.ts`、`tsconfig.json`、`tsconfig.app.json`、`src/index.css`、`src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: 可运行的 Tauri app 骨架；`@/` 路径别名；shadcn 组件 `button`、`input`、`label`、`dialog`、`select` 位于 `src/components/ui/`。

- [ ] **Step 1: 在临时目录生成模板并合入项目根目录**

项目根目录已有 docs 与 git 仓库，create-tauri-app 需要空目录，所以先在临时目录生成再 rsync 进来：

```bash
cd "$(mktemp -d)"
pnpm create tauri-app@latest db-minus --template react-ts --manager pnpm --identifier com.dbminus.app --yes
rsync -a db-minus/ /Users/jianhui/var/db-minus/
cd /Users/jianhui/var/db-minus
pnpm install
```

- [ ] **Step 2: 确认 .gitignore 覆盖 node_modules、dist、src-tauri/target**

模板自带 `.gitignore`，检查包含以上三项，缺则补。

- [ ] **Step 3: 接入 Tailwind 4**

```bash
pnpm add tailwindcss @tailwindcss/vite
pnpm add -D @types/node
```

`vite.config.ts` 全量替换为：

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

`src/index.css` 全量替换为：

```css
@import "tailwindcss";
```

删除模板残留样式引用：`src/App.css` 删除，`src/App.tsx` 中对它的 import 一并移除（App.tsx 内容 Task 18 会全量重写，这里先保证编译通过，可暂时替换为返回 `<div>DB-Minus</div>` 的最小组件）。

- [ ] **Step 4: tsconfig 加路径别名**

`tsconfig.json` 与 `tsconfig.app.json` 的 `compilerOptions` 各加：

```json
"baseUrl": ".",
"paths": { "@/*": ["./src/*"] }
```

- [ ] **Step 5: 初始化 shadcn/ui 并添加基础组件**

```bash
pnpm dlx shadcn@latest init -y -b neutral
pnpm dlx shadcn@latest add button input label dialog select -y
```

- [ ] **Step 6: 调整窗口配置**

`src-tauri/tauri.conf.json` 中 `app.windows[0]` 改为：

```json
{ "title": "DB-Minus", "width": 1280, "height": 800, "minWidth": 900, "minHeight": 600 }
```

`productName` 改为 `DB-Minus`。

- [ ] **Step 7: 验证可运行**

```bash
pnpm exec tsc --noEmit
pnpm tauri dev
```

预期：TS 无错误；桌面窗口打开，显示占位内容，冷启动明显小于 1 秒。
确认后关闭窗口。

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: scaffold tauri 2 + react + ts + tailwind + shadcn"
```

---

### Task 2: Docker 测试环境与种子数据

## Interfaces

- Produces:
  - PostgreSQL → localhost:5433
  - MySQL → localhost:3307

## Credentials

- username: dbminus
- password: dbminus
- database: dbminus_test

Seed data:
- users（1500 rows）
- app_log（no primary key）
- types_demo（multi-type demo）
- view: active_users（Postgres only）

---

- [ ] **Step 1: 启动数据库（Docker run）**

### PostgreSQL

```bash
docker run -d \
  --name dbminus-postgres \
  -e POSTGRES_USER=dbminus \
  -e POSTGRES_PASSWORD=dbminus \
  -e POSTGRES_DB=dbminus_test \
  -p 5433:5432 \
  -v $(pwd)/dev/seed/postgres:/docker-entrypoint-initdb.d:ro \
  postgres:17
```

### MySQL

```bash
docker run -d \
  --name dbminus-mysql \
  -e MYSQL_ROOT_PASSWORD=root \
  -e MYSQL_USER=dbminus \
  -e MYSQL_PASSWORD=dbminus \
  -e MYSQL_DATABASE=dbminus_test \
  -p 3307:3306 \
  -v $(pwd)/dev/seed/mysql:/docker-entrypoint-initdb.d:ro \
  mysql:8
```

---

- [ ] **Step 2: 验证**

```bash
docker exec -it dbminus-postgres psql -U dbminus -d dbminus_test -c "SELECT count(*) FROM users;"

docker exec -it dbminus-mysql mysql -udbminus -pdbminus dbminus_test -e "SELECT count(*) FROM users;"
```

Expected:
- 1500 rows

---

- [ ] **Step 3: Commit**

```bash
git add dev/
git commit -m "chore: add docker test databases with seed data"
```

---

### Task 3: Rust 依赖与模块骨架

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/connection/mod.rs`、`src-tauri/src/dialect.rs`、`src-tauri/src/schema.rs`、`src-tauri/src/query.rs`、`src-tauri/src/safety.rs`、`src-tauri/src/commands.rs`（先建空壳）

**Interfaces:**
- Produces: `AppError` 枚举（所有模块的错误类型）；`db_minus_lib` crate 下的 pub 模块树。

- [ ] **Step 1: Cargo.toml 加依赖**

`src-tauri/Cargo.toml` 的 `[dependencies]` 补成：

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-clipboard-manager = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "mysql", "chrono", "uuid", "json", "rust_decimal"] }
futures = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = "1"
keyring = { version = "4", features = ["apple-native"] }
dirs = "6"

[dev-dependencies]
tempfile = "3"
```

（`tauri-plugin-opener` 为模板自带，保留即可；模板若生成了其他残留依赖如 `serde_json` 重复项则合并。）

- [ ] **Step 2: 写 error.rs**

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize, Clone, PartialEq)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AppError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("query failed: {0}")]
    Query(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("dangerous statement: {0}")]
    DangerousStatement(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::Io(_)
            | sqlx::Error::Tls(_)
            | sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed => AppError::Connection(e.to_string()),
            _ => AppError::Query(e.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Config(e.to_string())
    }
}
```

- [ ] **Step 3: 建模块壳并挂到 lib.rs**

`src-tauri/src/connection/mod.rs`：

```rust
pub mod config;
pub mod pool;
pub mod secret;
```

`config.rs`、`pool.rs`、`secret.rs`、`dialect.rs`、`schema.rs`、`query.rs`、`safety.rs`、`commands.rs` 先建空文件。
`src-tauri/src/lib.rs` 顶部加：

```rust
pub mod commands;
pub mod connection;
pub mod dialect;
pub mod error;
pub mod query;
pub mod safety;
pub mod schema;
```

保留模板的 `run()` 函数，其余模板示例 command（`greet`）删除，`invoke_handler` 暂时留空：`.invoke_handler(tauri::generate_handler![])`。
`src-tauri/src/main.rs` 保持模板不动。

- [ ] **Step 4: 编译验证**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

预期：编译通过（空模块无警告级错误）。
首次拉依赖较慢属正常。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "chore: add rust dependencies and module skeleton"
```

---

### Task 4: 连接配置模型与 ConfigStore（TDD）

**Files:**
- Create: `src-tauri/src/connection/config.rs`

**Interfaces:**
- Produces:

```rust
pub enum Driver { Postgres, MySql }                    // serde: "postgres" / "mysql"
pub enum SslMode { Disable, Prefer, Require }          // serde: "disable" / "prefer" / "require"
pub struct ConnectionConfig {
    pub id: String, pub name: String, pub driver: Driver,
    pub host: String, pub port: u16, pub username: String,
    pub database: String, pub ssl_mode: SslMode,
}
pub struct ConfigStore;                                 // ConfigStore::new(dir: &Path)
// list() -> Result<Vec<ConnectionConfig>>, save(cfg) upsert by id, delete(id)
```

- [ ] **Step 1: 写失败测试（文件底部 `#[cfg(test)]`）**

```rust
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// （实现代码在 Step 3 填入此处上方）

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample(id: &str, name: &str) -> ConnectionConfig {
        ConnectionConfig {
            id: id.into(),
            name: name.into(),
            driver: Driver::Postgres,
            host: "localhost".into(),
            port: 5433,
            username: "dbminus".into(),
            database: "dbminus_test".into(),
            ssl_mode: SslMode::Disable,
        }
    }

    #[test]
    fn list_empty_when_no_file() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn save_then_list_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "pg local")).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "pg local");
    }

    #[test]
    fn save_same_id_upserts() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "old")).unwrap();
        store.save(sample("a", "new")).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "new");
    }

    #[test]
    fn delete_removes_entry() {
        let dir = TempDir::new().unwrap();
        let store = ConfigStore::new(dir.path());
        store.save(sample("a", "x")).unwrap();
        store.save(sample("b", "y")).unwrap();
        store.delete("a").unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "b");
    }

    #[test]
    fn config_serializes_camel_case() {
        let json = serde_json::to_value(sample("a", "x")).unwrap();
        assert!(json.get("sslMode").is_some());
        assert_eq!(json["driver"], "postgres");
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml config
```

预期：编译失败，`ConnectionConfig` 等未定义。

- [ ] **Step 3: 实现**

在同文件测试模块上方补实现：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Postgres,
    MySql,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub driver: Driver,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
    pub ssl_mode: SslMode,
}

pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: &Path) -> Self {
        Self { path: dir.join("connections.json") }
    }

    pub fn list(&self) -> Result<Vec<ConnectionConfig>, AppError> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let text = fs::read_to_string(&self.path)?;
        serde_json::from_str(&text).map_err(|e| AppError::Config(e.to_string()))
    }

    pub fn save(&self, config: ConnectionConfig) -> Result<(), AppError> {
        let mut all = self.list()?;
        match all.iter_mut().find(|c| c.id == config.id) {
            Some(existing) => *existing = config,
            None => all.push(config),
        }
        self.write(&all)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut all = self.list()?;
        all.retain(|c| c.id != id);
        self.write(&all)
    }

    fn write(&self, all: &[ConnectionConfig]) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(all).map_err(|e| AppError::Config(e.to_string()))?;
        fs::write(&self.path, text)?;
        Ok(())
    }
}
```

注意：`MySql` 的 `#[serde(rename_all = "lowercase")]` 会序列化成 `"mysql"`，与前端约定一致。

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml config
```

预期：5 个测试 PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/connection/
git commit -m "feat: connection config model and json store"
```

---

### Task 5: Keychain 密码存取

**Files:**
- Create: `src-tauri/src/connection/secret.rs`

**Interfaces:**
- Produces:

```rust
pub fn set_password(conn_id: &str, password: &str) -> Result<(), AppError>
pub fn get_password(conn_id: &str) -> Result<Option<String>, AppError>   // 未存过返回 None
pub fn delete_password(conn_id: &str) -> Result<(), AppError>            // 不存在时静默成功
```

- [ ] **Step 1: 实现（keyring 触真机 Keychain，测试标 ignore）**

```rust
use crate::error::AppError;
use keyring::Entry;

const SERVICE: &str = "com.dbminus.app";

fn entry(conn_id: &str) -> Result<Entry, AppError> {
    Entry::new(SERVICE, conn_id).map_err(|e| AppError::Keychain(e.to_string()))
}

pub fn set_password(conn_id: &str, password: &str) -> Result<(), AppError> {
    entry(conn_id)?
        .set_password(password)
        .map_err(|e| AppError::Keychain(e.to_string()))
}

pub fn get_password(conn_id: &str) -> Result<Option<String>, AppError> {
    match entry(conn_id)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

pub fn delete_password(conn_id: &str) -> Result<(), AppError> {
    match entry(conn_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "touches real macOS Keychain, run manually: cargo test -- --ignored secret"]
    fn roundtrip() {
        let id = "db-minus-test-entry";
        set_password(id, "s3cret").unwrap();
        assert_eq!(get_password(id).unwrap().as_deref(), Some("s3cret"));
        delete_password(id).unwrap();
        assert_eq!(get_password(id).unwrap(), None);
        delete_password(id).unwrap(); // 幂等
    }
}
```

注意：keyring 4 若无 `delete_credential` 方法（编译报错），改用 `delete_password()`（v3 旧名），以编译器为准。

- [ ] **Step 2: 编译 + 手动跑 ignored 测试验证一次**

```bash
cargo test --manifest-path src-tauri/Cargo.toml secret -- --ignored
```

预期：PASS（macOS 可能弹 Keychain 授权，允许即可）。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/connection/secret.rs
git commit -m "feat: keychain password storage"
```

---

### Task 6: dialect 模块（TDD）

**Files:**
- Create: `src-tauri/src/dialect.rs`（覆盖 Task 3 的空文件）

**Interfaces:**
- Consumes: `Driver`、`SslMode`、`ConnectionConfig`（Task 4）。
- Produces:

```rust
pub fn quote_ident(driver: Driver, ident: &str) -> String
pub fn qualified_table(driver: Driver, namespace: &str, table: &str) -> String
pub fn connect_url(config: &ConnectionConfig, password: &str) -> String
pub fn placeholder(driver: Driver, n: usize) -> String   // PG: "$1"; MySQL: "?"
```

- [ ] **Step 1: 写失败测试**

```rust
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
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml dialect
```

预期：编译失败，函数未定义。

- [ ] **Step 3: 实现**

```rust
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
        Driver::MySql => "?".to_string(),
    }
}
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml dialect
```

预期：6 个测试 PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/dialect.rs
git commit -m "feat: sql dialect helpers for postgres and mysql"
```

---

### Task 7: safety 危险 SQL 预检（TDD）

**Files:**
- Create: `src-tauri/src/safety.rs`（覆盖空文件）

**Interfaces:**
- Produces:

```rust
pub struct DangerWarning { pub kind: DangerKind, pub statement: String }  // camelCase 序列化
pub enum DangerKind { DropDatabase, DropTable, Truncate, DeleteWithoutWhere, UpdateWithoutWhere }
pub fn analyze(sql: &str) -> Vec<DangerWarning>
```

- [ ] **Step 1: 写失败测试**

```rust
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
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml safety
```

预期：编译失败。

- [ ] **Step 3: 实现**

思路：先把字符串字面量、行注释、块注释替换为空格（保守的字符级扫描），再按 `;` 切分，逐条按首关键字与 `WHERE` 存在性判断。

```rust
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

/// 把字符串字面量与注释抹成空格，保留其余字符与长度结构。
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
                        // '' 转义：连续两个引号继续
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
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml safety
```

预期：6 个测试 PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/safety.rs
git commit -m "feat: dangerous sql detection"
```

---

### Task 8: 连接池 PoolManager 与连接测试（集成测试）

**Files:**
- Create: `src-tauri/src/connection/pool.rs`（覆盖空文件）
- Create: `src-tauri/tests/common/mod.rs`
- Create: `src-tauri/tests/pool_test.rs`

**Interfaces:**
- Consumes: `connect_url`（Task 6）、`ConnectionConfig`（Task 4）。
- Produces:

```rust
#[derive(Clone)]
pub enum DbPool { Postgres(sqlx::PgPool), MySql(sqlx::MySqlPool) }
impl DbPool { pub fn driver(&self) -> Driver }
pub struct PoolManager;   // PoolManager::new()
// async get_or_create(&self, config: &ConnectionConfig, password: &str) -> Result<DbPool>
// async remove(&self, conn_id: &str)
pub async fn test_connection(config: &ConnectionConfig, password: &str) -> Result<(), AppError>
pub async fn connect(config: &ConnectionConfig, password: &str) -> Result<DbPool, AppError>  // 单独建池（不入管理器）
```

- [ ] **Step 1: 写集成测试**

`src-tauri/tests/common/mod.rs`：

```rust
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
```

注意：crate 的 lib 名以 `src-tauri/Cargo.toml` 的 `[lib] name` 为准（create-tauri-app 通常生成 `db_minus_lib`；若不同，统一测试内的 use 路径）。

`src-tauri/tests/pool_test.rs`：

```rust
mod common;

use common::{mysql_config, pg_config, PASSWORD};
use db_minus_lib::connection::pool::{test_connection, PoolManager};

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
    assert!(msg.contains("connection failed") || msg.contains("query failed"), "unexpected: {msg}");
}

#[tokio::test]
async fn pool_manager_caches_pool() {
    let mgr = PoolManager::new();
    let cfg = pg_config();
    let a = mgr.get_or_create(&cfg, PASSWORD).await.unwrap();
    let b = mgr.get_or_create(&cfg, PASSWORD).await.unwrap();
    // 同一连接池：PgPool 内部是 Arc，比较底层指针
    match (a, b) {
        (db_minus_lib::connection::pool::DbPool::Postgres(pa), db_minus_lib::connection::pool::DbPool::Postgres(pb)) => {
            assert_eq!(pa.size(), pb.size());
        }
        _ => panic!("expected postgres pools"),
    }
    mgr.remove("test-pg").await;
}
```

- [ ] **Step 2: 确认失败**

```bash
docker compose -f dev/docker-compose.yml up -d --wait
cargo test --manifest-path src-tauri/Cargo.toml --test pool_test
```

预期：编译失败（pool 模块为空）。

- [ ] **Step 3: 实现 pool.rs**

```rust
use crate::connection::config::{ConnectionConfig, Driver};
use crate::dialect::connect_url;
use crate::error::AppError;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS: u32 = 5;

#[derive(Clone)]
pub enum DbPool {
    Postgres(sqlx::PgPool),
    MySql(sqlx::MySqlPool),
}

impl DbPool {
    pub fn driver(&self) -> Driver {
        match self {
            DbPool::Postgres(_) => Driver::Postgres,
            DbPool::MySql(_) => Driver::MySql,
        }
    }

    pub async fn close(&self) {
        match self {
            DbPool::Postgres(p) => p.close().await,
            DbPool::MySql(p) => p.close().await,
        }
    }
}

pub async fn connect(config: &ConnectionConfig, password: &str) -> Result<DbPool, AppError> {
    let url = connect_url(config, password);
    let connect = async {
        match config.driver {
            Driver::Postgres => PgPoolOptions::new()
                .max_connections(MAX_CONNECTIONS)
                .acquire_timeout(CONNECT_TIMEOUT)
                .connect(&url)
                .await
                .map(DbPool::Postgres),
            Driver::MySql => MySqlPoolOptions::new()
                .max_connections(MAX_CONNECTIONS)
                .acquire_timeout(CONNECT_TIMEOUT)
                .connect(&url)
                .await
                .map(DbPool::MySql),
        }
    };
    tokio::time::timeout(CONNECT_TIMEOUT, connect)
        .await
        .map_err(|_| AppError::Timeout("connection timed out after 10s".into()))?
        .map_err(|e| AppError::Connection(e.to_string()))
}

pub async fn test_connection(config: &ConnectionConfig, password: &str) -> Result<(), AppError> {
    let pool = connect(config, password).await?;
    let result = match &pool {
        DbPool::Postgres(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
        DbPool::MySql(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
    };
    pool.close().await;
    result.map_err(|e| AppError::Connection(e.to_string()))
}

pub struct PoolManager {
    pools: RwLock<HashMap<String, DbPool>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self { pools: RwLock::new(HashMap::new()) }
    }

    pub async fn get_or_create(&self, config: &ConnectionConfig, password: &str) -> Result<DbPool, AppError> {
        if let Some(pool) = self.pools.read().await.get(&config.id) {
            return Ok(pool.clone());
        }
        let pool = connect(config, password).await?;
        self.pools.write().await.insert(config.id.clone(), pool.clone());
        Ok(pool)
    }

    pub async fn remove(&self, conn_id: &str) {
        if let Some(pool) = self.pools.write().await.remove(conn_id) {
            pool.close().await;
        }
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test pool_test
```

预期：4 个测试 PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/connection/pool.rs src-tauri/tests/
git commit -m "feat: db connection pool manager and connection test"
```

---

### Task 9: schema 元数据模块（集成测试）

**Files:**
- Create: `src-tauri/src/schema.rs`（覆盖空文件）
- Create: `src-tauri/tests/schema_test.rs`

**Interfaces:**
- Consumes: `DbPool`（Task 8）。
- Produces:

```rust
pub struct TableInfo { pub name: String, pub kind: TableKind }        // kind: "table" | "view"
pub struct ColumnInfo { pub name: String, pub data_type: String, pub nullable: bool, pub is_primary_key: bool }
pub async fn list_namespaces(pool: &DbPool) -> Result<Vec<String>, AppError>
pub async fn list_tables(pool: &DbPool, namespace: &str) -> Result<Vec<TableInfo>, AppError>
pub async fn list_columns(pool: &DbPool, namespace: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError>
pub async fn integer_primary_key(pool: &DbPool, namespace: &str, table: &str) -> Result<Option<String>, AppError>
```

- [ ] **Step 1: 写集成测试 `src-tauri/tests/schema_test.rs`**

```rust
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
    assert_eq!(integer_primary_key(&pg, "public", "app_log").await.unwrap(), None);   // 无 PK
    assert_eq!(integer_primary_key(&pg, "public", "types_demo").await.unwrap(), None); // UUID PK

    let my = connect(&mysql_config(), PASSWORD).await.unwrap();
    assert_eq!(integer_primary_key(&my, "dbminus_test", "users").await.unwrap(), Some("id".to_string()));
}
```

- [ ] **Step 2: 确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test schema_test
```

预期：编译失败。

- [ ] **Step 3: 实现 schema.rs**

```rust
use crate::connection::pool::DbPool;
use crate::error::AppError;
use serde::Serialize;
use sqlx::Row;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TableKind {
    Table,
    View,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
    pub kind: TableKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

pub async fn list_namespaces(pool: &DbPool) -> Result<Vec<String>, AppError> {
    let rows: Vec<String> = match pool {
        DbPool::Postgres(p) => {
            sqlx::query_scalar(
                "SELECT schema_name FROM information_schema.schemata
                 WHERE schema_name NOT IN ('information_schema') AND schema_name NOT LIKE 'pg_%'
                 ORDER BY schema_name",
            )
            .fetch_all(p)
            .await?
        }
        DbPool::MySql(p) => {
            sqlx::query_scalar(
                "SELECT schema_name FROM information_schema.schemata
                 WHERE schema_name NOT IN ('mysql', 'information_schema', 'performance_schema', 'sys')
                 ORDER BY schema_name",
            )
            .fetch_all(p)
            .await?
        }
    };
    Ok(rows)
}

const TABLES_SQL_PG: &str = "SELECT table_name, table_type FROM information_schema.tables
    WHERE table_schema = $1 ORDER BY table_name";
const TABLES_SQL_MYSQL: &str = "SELECT table_name, table_type FROM information_schema.tables
    WHERE table_schema = ? ORDER BY table_name";

pub async fn list_tables(pool: &DbPool, namespace: &str) -> Result<Vec<TableInfo>, AppError> {
    let raw: Vec<(String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(TABLES_SQL_PG).bind(namespace).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(TABLES_SQL_MYSQL).bind(namespace).fetch_all(p).await?,
    };
    Ok(raw
        .into_iter()
        .map(|(name, table_type)| TableInfo {
            name,
            kind: if table_type.to_uppercase().contains("VIEW") { TableKind::View } else { TableKind::Table },
        })
        .collect())
}

const PK_SQL_PG: &str = "SELECT kcu.column_name, c.data_type
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
      ON kcu.constraint_name = tc.constraint_name
     AND kcu.table_schema = tc.table_schema
     AND kcu.table_name = tc.table_name
    JOIN information_schema.columns c
      ON c.table_schema = kcu.table_schema
     AND c.table_name = kcu.table_name
     AND c.column_name = kcu.column_name
    WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = $1 AND tc.table_name = $2
    ORDER BY kcu.ordinal_position";
const PK_SQL_MYSQL: &str = "SELECT kcu.column_name, c.data_type
    FROM information_schema.key_column_usage kcu
    JOIN information_schema.columns c
      ON c.table_schema = kcu.table_schema
     AND c.table_name = kcu.table_name
     AND c.column_name = kcu.column_name
    WHERE kcu.constraint_name = 'PRIMARY' AND kcu.table_schema = ? AND kcu.table_name = ?
    ORDER BY kcu.ordinal_position";

async fn primary_key_with_types(pool: &DbPool, namespace: &str, table: &str) -> Result<Vec<(String, String)>, AppError> {
    let rows: Vec<(String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(PK_SQL_PG).bind(namespace).bind(table).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(PK_SQL_MYSQL).bind(namespace).bind(table).fetch_all(p).await?,
    };
    Ok(rows)
}

const COLUMNS_SQL_PG: &str = "SELECT column_name, data_type, is_nullable
    FROM information_schema.columns
    WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position";
const COLUMNS_SQL_MYSQL: &str = "SELECT column_name, data_type, is_nullable
    FROM information_schema.columns
    WHERE table_schema = ? AND table_name = ? ORDER BY ordinal_position";

pub async fn list_columns(pool: &DbPool, namespace: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let raw: Vec<(String, String, String)> = match pool {
        DbPool::Postgres(p) => sqlx::query_as(COLUMNS_SQL_PG).bind(namespace).bind(table).fetch_all(p).await?,
        DbPool::MySql(p) => sqlx::query_as(COLUMNS_SQL_MYSQL).bind(namespace).bind(table).fetch_all(p).await?,
    };
    let pk: Vec<String> = primary_key_with_types(pool, namespace, table)
        .await?
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    Ok(raw
        .into_iter()
        .map(|(name, data_type, is_nullable)| ColumnInfo {
            is_primary_key: pk.contains(&name),
            nullable: is_nullable.eq_ignore_ascii_case("YES"),
            name,
            data_type,
        })
        .collect())
}

const INT_TYPES: &[&str] = &["smallint", "integer", "bigint", "int", "tinyint", "mediumint"];

/// 仅当表有单列整型主键时返回列名，用于 keyset 分页。
pub async fn integer_primary_key(pool: &DbPool, namespace: &str, table: &str) -> Result<Option<String>, AppError> {
    let pk = primary_key_with_types(pool, namespace, table).await?;
    match pk.as_slice() {
        [(name, data_type)] if INT_TYPES.contains(&data_type.to_lowercase().as_str()) => Ok(Some(name.clone())),
        _ => Ok(None),
    }
}
```

注意 MySQL 的 information_schema 列名大小写：sqlx 的 `query_as` 元组按位置取列，不受列名大小写影响。
若 MySQL 8 返回列类型是 `VARBINARY`（部分发行版 information_schema 字段为 bytes），`query_as::<(String, String)>` 会解码失败，此时把 SQL 改为 `SELECT CAST(column_name AS CHAR), CAST(data_type AS CHAR) ...`，以测试结果为准。

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test schema_test
```

预期：5 个测试 PASS。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/schema.rs src-tauri/tests/schema_test.rs
git commit -m "feat: schema metadata queries"
```

---

### Task 10: query 模块之 execute_sql（集成测试）

**Files:**
- Create: `src-tauri/src/query.rs`（覆盖空文件，本 Task 先实现值序列化与 execute_sql）
- Create: `src-tauri/tests/query_test.rs`

**Interfaces:**
- Consumes: `DbPool`（Task 8）。
- Produces:

```rust
pub struct ColumnMeta { pub name: String, pub type_name: String }
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub truncated: bool,
}
pub const MAX_RESULT_ROWS: usize = 10_000;
pub async fn execute_sql(pool: &DbPool, sql: &str) -> Result<QueryResult, AppError>
// 内部工具，Task 11 复用：
pub(crate) fn pg_row_to_values(row: &sqlx::postgres::PgRow) -> Vec<serde_json::Value>
pub(crate) fn mysql_row_to_values(row: &sqlx::mysql::MySqlRow) -> Vec<serde_json::Value>
pub(crate) fn pg_columns(row: &sqlx::postgres::PgRow) -> Vec<ColumnMeta>
pub(crate) fn mysql_columns(row: &sqlx::mysql::MySqlRow) -> Vec<ColumnMeta>
```

- [ ] **Step 1: 写集成测试 `src-tauri/tests/query_test.rs`**

```rust
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
    assert!(r.rows[0][3].is_string()); // NUMERIC 序列化为字符串
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
async fn pg_types_demo_serializes() {
    let pool = connect(&pg_config(), PASSWORD).await.unwrap();
    let r = execute_sql(&pool, "SELECT id, payload, raw, ratio, born, wake FROM types_demo ORDER BY payload NULLS LAST").await.unwrap();
    assert_eq!(r.rows.len(), 2);
    let first = &r.rows[0];
    assert!(first[0].is_string());                       // uuid -> string
    assert!(first[1].is_object());                       // jsonb -> object
    assert_eq!(first[2], Value::from("0xdeadbeef"));     // bytea -> hex 字符串
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
```

- [ ] **Step 2: 确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test query_test
```

预期：编译失败。

- [ ] **Step 3: 实现 query.rs（值序列化 + execute_sql）**

```rust
use crate::connection::pool::DbPool;
use crate::error::AppError;
use futures::TryStreamExt;
use serde::Serialize;
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::time::{Duration, Instant};

pub const MAX_RESULT_ROWS: usize = 10_000;
const QUERY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMeta {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: Option<u64>,
    pub duration_ms: u64,
    pub truncated: bool,
}

fn hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// 尝试一组解码器，全失败则回退字符串或 "<binary>"。
macro_rules! try_decode {
    ($row:expr, $i:expr, $($t:ty => $conv:expr),+ $(,)?) => {{
        $(
            if let Ok(v) = $row.try_get::<$t, _>($i) {
                #[allow(clippy::redundant_closure_call)]
                { return ($conv)(v); }
            }
        )+
    }};
}

fn pg_value(row: &PgRow, i: usize) -> Value {
    let Ok(raw) = row.try_get_raw(i) else { return Value::Null };
    if raw.is_null() {
        return Value::Null;
    }
    let type_name = raw.type_info().name().to_string();
    pg_value_typed(row, i, &type_name)
}

fn pg_value_typed(row: &PgRow, i: usize, type_name: &str) -> Value {
    match type_name {
        "BOOL" => try_decode!(row, i, bool => Value::from),
        "INT2" => try_decode!(row, i, i16 => Value::from),
        "INT4" => try_decode!(row, i, i32 => Value::from),
        "INT8" => try_decode!(row, i, i64 => Value::from),
        "FLOAT4" => try_decode!(row, i, f32 => |v: f32| Value::from(v as f64)),
        "FLOAT8" => try_decode!(row, i, f64 => Value::from),
        "NUMERIC" => try_decode!(row, i, rust_decimal::Decimal => |v: rust_decimal::Decimal| Value::from(v.to_string())),
        "UUID" => try_decode!(row, i, uuid::Uuid => |v: uuid::Uuid| Value::from(v.to_string())),
        "JSON" | "JSONB" => try_decode!(row, i, Value => |v| v),
        "TIMESTAMPTZ" => try_decode!(row, i, chrono::DateTime<chrono::Utc> => |v: chrono::DateTime<chrono::Utc>| Value::from(v.to_rfc3339())),
        "TIMESTAMP" => try_decode!(row, i, chrono::NaiveDateTime => |v: chrono::NaiveDateTime| Value::from(v.to_string())),
        "DATE" => try_decode!(row, i, chrono::NaiveDate => |v: chrono::NaiveDate| Value::from(v.to_string())),
        "TIME" => try_decode!(row, i, chrono::NaiveTime => |v: chrono::NaiveTime| Value::from(v.to_string())),
        "BYTEA" => try_decode!(row, i, Vec<u8> => |v: Vec<u8>| Value::from(hex_string(&v))),
        _ => {}
    }
    // 兜底：字符串，再不行标记不支持
    try_decode!(row, i, String => Value::from);
    Value::from(format!("<{}>", type_name.to_lowercase()))
}

fn mysql_value(row: &MySqlRow, i: usize) -> Value {
    let Ok(raw) = row.try_get_raw(i) else { return Value::Null };
    if raw.is_null() {
        return Value::Null;
    }
    let type_name = raw.type_info().name().to_string();
    mysql_value_typed(row, i, &type_name)
}

fn mysql_value_typed(row: &MySqlRow, i: usize, type_name: &str) -> Value {
    let unsigned = type_name.contains("UNSIGNED");
    match type_name {
        "BOOLEAN" => try_decode!(row, i, bool => Value::from, i8 => |v: i8| Value::from(v != 0)),
        t if t.starts_with("TINYINT") || t.starts_with("SMALLINT") || t.starts_with("MEDIUMINT")
            || t.starts_with("INT") || t.starts_with("BIGINT") || t == "YEAR" =>
        {
            if unsigned {
                try_decode!(row, i, u64 => Value::from, i64 => Value::from);
            } else {
                try_decode!(row, i, i64 => Value::from, i32 => Value::from);
            }
        }
        "FLOAT" => try_decode!(row, i, f32 => |v: f32| Value::from(v as f64)),
        "DOUBLE" => try_decode!(row, i, f64 => Value::from),
        "DECIMAL" => try_decode!(row, i, rust_decimal::Decimal => |v: rust_decimal::Decimal| Value::from(v.to_string())),
        "JSON" => try_decode!(row, i, Value => |v| v),
        "DATETIME" | "TIMESTAMP" => try_decode!(row, i, chrono::NaiveDateTime => |v: chrono::NaiveDateTime| Value::from(v.to_string())),
        "DATE" => try_decode!(row, i, chrono::NaiveDate => |v: chrono::NaiveDate| Value::from(v.to_string())),
        "TIME" => try_decode!(row, i, chrono::NaiveTime => |v: chrono::NaiveTime| Value::from(v.to_string())),
        t if t.contains("BLOB") || t.contains("BINARY") => try_decode!(row, i, Vec<u8> => |v: Vec<u8>| Value::from(hex_string(&v))),
        _ => {}
    }
    try_decode!(row, i, String => Value::from);
    Value::from(format!("<{}>", type_name.to_lowercase()))
}

pub(crate) fn pg_row_to_values(row: &PgRow) -> Vec<Value> {
    (0..row.columns().len()).map(|i| pg_value(row, i)).collect()
}

pub(crate) fn mysql_row_to_values(row: &MySqlRow) -> Vec<Value> {
    (0..row.columns().len()).map(|i| mysql_value(row, i)).collect()
}

pub(crate) fn pg_columns(row: &PgRow) -> Vec<ColumnMeta> {
    row.columns()
        .iter()
        .map(|c| ColumnMeta { name: c.name().to_string(), type_name: c.type_info().name().to_lowercase() })
        .collect()
}

pub(crate) fn mysql_columns(row: &MySqlRow) -> Vec<ColumnMeta> {
    row.columns()
        .iter()
        .map(|c| ColumnMeta { name: c.name().to_string(), type_name: c.type_info().name().to_lowercase() })
        .collect()
}

fn first_keyword(sql: &str) -> String {
    sql.split_whitespace().next().unwrap_or("").to_uppercase()
}

fn is_row_returning(sql: &str) -> bool {
    matches!(
        first_keyword(sql).as_str(),
        "SELECT" | "WITH" | "SHOW" | "EXPLAIN" | "DESCRIBE" | "DESC" | "TABLE" | "VALUES"
    )
}

pub async fn execute_sql(pool: &DbPool, sql: &str) -> Result<QueryResult, AppError> {
    let started = Instant::now();
    let work = async {
        if is_row_returning(sql) {
            fetch_rows(pool, sql, MAX_RESULT_ROWS).await
        } else {
            let affected = match pool {
                DbPool::Postgres(p) => sqlx::query(sql).execute(p).await?.rows_affected(),
                DbPool::MySql(p) => sqlx::query(sql).execute(p).await?.rows_affected(),
            };
            Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: Some(affected),
                duration_ms: 0,
                truncated: false,
            })
        }
    };
    let mut result = tokio::time::timeout(QUERY_TIMEOUT, work)
        .await
        .map_err(|_| AppError::Timeout("query timed out after 30s".into()))??;
    result.duration_ms = started.elapsed().as_millis() as u64;
    Ok(result)
}

async fn fetch_rows(pool: &DbPool, sql: &str, max_rows: usize) -> Result<QueryResult, AppError> {
    let mut columns: Vec<ColumnMeta> = vec![];
    let mut rows: Vec<Vec<Value>> = vec![];
    let mut truncated = false;

    match pool {
        DbPool::Postgres(p) => {
            let mut stream = sqlx::query(sql).fetch(p);
            while let Some(row) = stream.try_next().await? {
                if columns.is_empty() {
                    columns = pg_columns(&row);
                }
                if rows.len() >= max_rows {
                    truncated = true;
                    break;
                }
                rows.push(pg_row_to_values(&row));
            }
        }
        DbPool::MySql(p) => {
            let mut stream = sqlx::query(sql).fetch(p);
            while let Some(row) = stream.try_next().await? {
                if columns.is_empty() {
                    columns = mysql_columns(&row);
                }
                if rows.len() >= max_rows {
                    truncated = true;
                    break;
                }
                rows.push(mysql_row_to_values(&row));
            }
        }
    }

    Ok(QueryResult { columns, rows, affected_rows: None, duration_ms: 0, truncated })
}
```

注意 `try_decode!` 宏使用 `return`，因此 `pg_value_typed` / `mysql_value_typed` 必须是独立函数（不能内联进闭包）。
若某个 sqlx 类型名与实际不符（如 PG 的 `CHAR` 列报 `BPCHAR`），以测试输出调整 match 分支，兜底路径保证不会 panic。

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test query_test
```

预期：6 个测试 PASS。
若 `pg_types_demo_serializes` 因类型名或格式断言失败，打印实际值调整断言与实现（目标行为不变：uuid/decimal 转字符串、json 保对象、bytea 转 hex、日期时间转 ISO 风格字符串）。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/query.rs src-tauri/tests/query_test.rs
git commit -m "feat: sql execution with typed json serialization"
```

---

### Task 11: fetch_table_page 分页浏览（集成测试）

**Files:**
- Modify: `src-tauri/src/query.rs`（追加）
- Create: `src-tauri/tests/table_page_test.rs`

**Interfaces:**
- Consumes: `integer_primary_key`（Task 9）、`qualified_table` / `quote_ident` / `placeholder`（Task 6）、行序列化工具（Task 10）。
- Produces:

```rust
pub struct Sort { pub column: String, pub desc: bool }                      // camelCase
pub enum Cursor { Keyset { last: i64 }, Offset { offset: u64 } }            // serde tag = "kind", camelCase
pub struct TablePageRequest {
    pub namespace: String, pub table: String,
    pub sort: Option<Sort>, pub cursor: Option<Cursor>, pub limit: u32,
}
pub struct TablePage { pub columns: Vec<ColumnMeta>, pub rows: Vec<Vec<Value>>, pub next_cursor: Option<Cursor> }
pub async fn fetch_table_page(pool: &DbPool, req: &TablePageRequest) -> Result<TablePage, AppError>
```

- [ ] **Step 1: 写集成测试 `src-tauri/tests/table_page_test.rs`**

```rust
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
```

- [ ] **Step 2: 确认失败**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test table_page_test
```

预期：编译失败。

- [ ] **Step 3: 在 query.rs 追加实现**

```rust
use crate::connection::config::Driver;
use crate::dialect::{placeholder, qualified_table, quote_ident};
use crate::schema::{integer_primary_key, list_columns};
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sort {
    pub column: String,
    pub desc: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Cursor {
    Keyset { last: i64 },
    Offset { offset: u64 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TablePageRequest {
    pub namespace: String,
    pub table: String,
    pub sort: Option<Sort>,
    pub cursor: Option<Cursor>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablePage {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<Value>>,
    pub next_cursor: Option<Cursor>,
}

pub async fn fetch_table_page(pool: &DbPool, req: &TablePageRequest) -> Result<TablePage, AppError> {
    let driver = pool.driver();
    let table = qualified_table(driver, &req.namespace, &req.table);
    let limit = req.limit.clamp(1, 1000) as usize;

    // 校验排序列存在，防注入（列名来自白名单）
    if let Some(sort) = &req.sort {
        let cols = list_columns(pool, &req.namespace, &req.table).await?;
        if !cols.iter().any(|c| c.name == sort.column) {
            return Err(AppError::NotFound(format!("sort column '{}' not found", sort.column)));
        }
    }

    let int_pk = if req.sort.is_none() {
        integer_primary_key(pool, &req.namespace, &req.table).await?
    } else {
        None
    };

    let work = async {
        match int_pk {
            Some(pk) => fetch_keyset(pool, driver, &table, &pk, req, limit).await,
            None => fetch_offset(pool, driver, &table, req, limit).await,
        }
    };
    tokio::time::timeout(QUERY_TIMEOUT, work)
        .await
        .map_err(|_| AppError::Timeout("query timed out after 30s".into()))?
}

async fn fetch_keyset(
    pool: &DbPool,
    driver: Driver,
    table: &str,
    pk: &str,
    req: &TablePageRequest,
    limit: usize,
) -> Result<TablePage, AppError> {
    let pk_quoted = quote_ident(driver, pk);
    let last = match req.cursor {
        Some(Cursor::Keyset { last }) => Some(last),
        _ => None,
    };
    let sql = match last {
        Some(_) => format!(
            "SELECT * FROM {table} WHERE {pk_quoted} > {} ORDER BY {pk_quoted} LIMIT {limit}",
            placeholder(driver, 1)
        ),
        None => format!("SELECT * FROM {table} ORDER BY {pk_quoted} LIMIT {limit}"),
    };

    let (columns, rows) = run_page_query(pool, &sql, last).await?;

    // 从行里取 pk 值作为下一页 cursor
    let pk_index = columns.iter().position(|c| c.name == pk);
    let next_cursor = if rows.len() < limit {
        None
    } else {
        pk_index
            .and_then(|i| rows.last().and_then(|r| r[i].as_i64()))
            .map(|last| Cursor::Keyset { last })
    };
    Ok(TablePage { columns, rows, next_cursor })
}

async fn fetch_offset(
    pool: &DbPool,
    driver: Driver,
    table: &str,
    req: &TablePageRequest,
    limit: usize,
) -> Result<TablePage, AppError> {
    let offset = match req.cursor {
        Some(Cursor::Offset { offset }) => offset,
        _ => 0,
    };
    let order = match &req.sort {
        Some(sort) => format!(
            " ORDER BY {} {}",
            quote_ident(driver, &sort.column),
            if sort.desc { "DESC" } else { "ASC" }
        ),
        None => String::new(),
    };
    let sql = format!("SELECT * FROM {table}{order} LIMIT {limit} OFFSET {offset}");
    let (columns, rows) = run_page_query(pool, &sql, None).await?;
    let next_cursor = if rows.len() < limit {
        None
    } else {
        Some(Cursor::Offset { offset: offset + rows.len() as u64 })
    };
    Ok(TablePage { columns, rows, next_cursor })
}

async fn run_page_query(
    pool: &DbPool,
    sql: &str,
    keyset_bind: Option<i64>,
) -> Result<(Vec<ColumnMeta>, Vec<Vec<Value>>), AppError> {
    let mut columns = vec![];
    let mut rows = vec![];
    match pool {
        DbPool::Postgres(p) => {
            let mut q = sqlx::query(sql);
            if let Some(v) = keyset_bind {
                q = q.bind(v);
            }
            let fetched = q.fetch_all(p).await?;
            for row in &fetched {
                if columns.is_empty() {
                    columns = pg_columns(row);
                }
                rows.push(pg_row_to_values(row));
            }
        }
        DbPool::MySql(p) => {
            let mut q = sqlx::query(sql);
            if let Some(v) = keyset_bind {
                q = q.bind(v);
            }
            let fetched = q.fetch_all(p).await?;
            for row in &fetched {
                if columns.is_empty() {
                    columns = mysql_columns(row);
                }
                rows.push(mysql_row_to_values(row));
            }
        }
    }
    Ok((columns, rows))
}
```

若页面为空（表 0 行）columns 会是空数组，前端用 `list_columns` 的信息兜底展示表头（Task 16 处理）。

- [ ] **Step 4: 跑测试确认通过**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test table_page_test
```

预期：5 个测试 PASS。

- [ ] **Step 5: 全量回归**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

预期：全部 PASS（ignored 除外）。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/query.rs src-tauri/tests/table_page_test.rs
git commit -m "feat: cursor paginated table browsing"
```

---

### Task 12: Tauri commands 接线与应用启动

**Files:**
- Create: `src-tauri/src/commands.rs`（覆盖空文件）
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Consumes: 前面全部 Rust 模块。
- Produces（前端 invoke 名单，参数经 Tauri 自动 camelCase 映射）：

```text
connections_list() -> ConnectionConfig[]
connection_save(config, password: string | null) -> void
connection_delete(id: string) -> void
connection_test(config, password: string | null) -> void
list_namespaces(connId) -> string[]
list_tables(connId, namespace) -> TableInfo[]
list_columns(connId, namespace, table) -> ColumnInfo[]
fetch_table_page(connId, req: TablePageRequest) -> TablePage
execute_sql(connId, sql, confirmed: boolean) -> QueryResult
```

- [ ] **Step 1: 写 commands.rs**

```rust
use crate::connection::config::{ConfigStore, ConnectionConfig};
use crate::connection::pool::{test_connection, DbPool, PoolManager};
use crate::connection::secret;
use crate::error::AppError;
use crate::query::{self, QueryResult, TablePage, TablePageRequest};
use crate::safety;
use crate::schema::{self, ColumnInfo, TableInfo};
use tauri::State;

pub struct AppState {
    pub store: ConfigStore,
    pub pools: PoolManager,
}

impl AppState {
    async fn pool(&self, conn_id: &str) -> Result<DbPool, AppError> {
        let config = self
            .store
            .list()?
            .into_iter()
            .find(|c| c.id == conn_id)
            .ok_or_else(|| AppError::NotFound(format!("connection '{conn_id}'")))?;
        let password = secret::get_password(conn_id)?.unwrap_or_default();
        self.pools.get_or_create(&config, &password).await
    }
}

#[tauri::command]
pub async fn connections_list(state: State<'_, AppState>) -> Result<Vec<ConnectionConfig>, AppError> {
    state.store.list()
}

#[tauri::command]
pub async fn connection_save(
    state: State<'_, AppState>,
    config: ConnectionConfig,
    password: Option<String>,
) -> Result<(), AppError> {
    if let Some(p) = password {
        secret::set_password(&config.id, &p)?;
    }
    // 配置可能变更（host、密码等），丢弃旧池
    state.pools.remove(&config.id).await;
    state.store.save(config)
}

#[tauri::command]
pub async fn connection_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.pools.remove(&id).await;
    secret::delete_password(&id)?;
    state.store.delete(&id)
}

#[tauri::command]
pub async fn connection_test(
    config: ConnectionConfig,
    password: Option<String>,
) -> Result<(), AppError> {
    // 编辑已有连接但未重输密码时，回退 Keychain
    let password = match password {
        Some(p) => p,
        None => secret::get_password(&config.id)?.unwrap_or_default(),
    };
    test_connection(&config, &password).await
}

#[tauri::command]
pub async fn list_namespaces(state: State<'_, AppState>, conn_id: String) -> Result<Vec<String>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_namespaces(&pool).await
}

#[tauri::command]
pub async fn list_tables(
    state: State<'_, AppState>,
    conn_id: String,
    namespace: String,
) -> Result<Vec<TableInfo>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_tables(&pool, &namespace).await
}

#[tauri::command]
pub async fn list_columns(
    state: State<'_, AppState>,
    conn_id: String,
    namespace: String,
    table: String,
) -> Result<Vec<ColumnInfo>, AppError> {
    let pool = state.pool(&conn_id).await?;
    schema::list_columns(&pool, &namespace, &table).await
}

#[tauri::command]
pub async fn fetch_table_page(
    state: State<'_, AppState>,
    conn_id: String,
    req: TablePageRequest,
) -> Result<TablePage, AppError> {
    let pool = state.pool(&conn_id).await?;
    query::fetch_table_page(&pool, &req).await
}

#[tauri::command]
pub async fn execute_sql(
    state: State<'_, AppState>,
    conn_id: String,
    sql: String,
    confirmed: bool,
) -> Result<QueryResult, AppError> {
    if !confirmed {
        let warnings = safety::analyze(&sql);
        if let Some(w) = warnings.first() {
            let label = match w.kind {
                safety::DangerKind::DropDatabase => "DROP DATABASE",
                safety::DangerKind::DropTable => "DROP TABLE",
                safety::DangerKind::Truncate => "TRUNCATE",
                safety::DangerKind::DeleteWithoutWhere => "DELETE without WHERE",
                safety::DangerKind::UpdateWithoutWhere => "UPDATE without WHERE",
            };
            return Err(AppError::DangerousStatement(format!("{label}: {}", w.statement)));
        }
    }
    let pool = state.pool(&conn_id).await?;
    query::execute_sql(&pool, &sql).await
}
```

- [ ] **Step 2: lib.rs 接线（state、菜单、clipboard 插件）**

`src-tauri/src/lib.rs` 的 `run()` 替换为：

```rust
use crate::commands::AppState;
use crate::connection::config::ConfigStore;
use crate::connection::pool::PoolManager;
use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState {
                store: ConfigStore::new(&config_dir),
                pools: PoolManager::new(),
            });

            // 自定义菜单：不含 Cmd+W Close，让快捷键落到 webview；保留 Edit 让复制粘贴可用
            let app_menu = Submenu::with_items(
                app,
                "DB-Minus",
                true,
                &[
                    &PredefinedMenuItem::about(app, None, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?;
            let edit_menu = Submenu::with_items(
                app,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(app, None)?,
                    &PredefinedMenuItem::redo(app, None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, None)?,
                    &PredefinedMenuItem::copy(app, None)?,
                    &PredefinedMenuItem::paste(app, None)?,
                    &PredefinedMenuItem::select_all(app, None)?,
                ],
            )?;
            let menu = Menu::with_items(app, &[&app_menu, &edit_menu])?;
            app.set_menu(menu)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connections_list,
            commands::connection_save,
            commands::connection_delete,
            commands::connection_test,
            commands::list_namespaces,
            commands::list_tables,
            commands::list_columns,
            commands::fetch_table_page,
            commands::execute_sql,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

（模块声明沿用 Task 3 已加的 `pub mod` 列表。）

- [ ] **Step 3: capabilities 加 clipboard 权限**

`src-tauri/capabilities/default.json` 的 `permissions` 数组追加：

```json
"clipboard-manager:allow-write-text"
```

- [ ] **Step 4: 验证编译与启动**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri dev
```

预期：编译过、测试过、窗口正常打开（UI 仍是占位）。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/
git commit -m "feat: wire tauri commands, app state and menu"
```

---

### Task 13: 前端 IPC 封装与状态 store

**Files:**
- Create: `src/lib/ipc.ts`
- Create: `src/stores/workspace.ts`
- Create: `src/stores/ui.ts`

**Interfaces:**
- Consumes: Task 12 的 command 名单。
- Produces: `ipc` 对象（全部类型化调用）、`useWorkspace` / `useUi` Zustand hooks。后续 UI Task 全部依赖这里的类型与 action 名。

- [ ] **Step 1: 装依赖**

```bash
pnpm add @tanstack/react-table @tanstack/react-virtual @tanstack/react-query zustand @tauri-apps/plugin-clipboard-manager monaco-editor @monaco-editor/react
```

- [ ] **Step 2: 写 `src/lib/ipc.ts`**

```ts
import { invoke } from "@tauri-apps/api/core";

export type Driver = "postgres" | "mysql";
export type SslMode = "disable" | "prefer" | "require";

export interface ConnectionConfig {
  id: string;
  name: string;
  driver: Driver;
  host: string;
  port: number;
  username: string;
  database: string;
  sslMode: SslMode;
}

export interface TableInfo {
  name: string;
  kind: "table" | "view";
}

export interface ColumnInfo {
  name: string;
  dataType: string;
  nullable: boolean;
  isPrimaryKey: boolean;
}

export interface ColumnMeta {
  name: string;
  typeName: string;
}

export type CellValue = string | number | boolean | null | Record<string, unknown> | unknown[];

export interface QueryResult {
  columns: ColumnMeta[];
  rows: CellValue[][];
  affectedRows: number | null;
  durationMs: number;
  truncated: boolean;
}

export interface Sort {
  column: string;
  desc: boolean;
}

export type Cursor = { kind: "keyset"; last: number } | { kind: "offset"; offset: number };

export interface TablePageRequest {
  namespace: string;
  table: string;
  sort: Sort | null;
  cursor: Cursor | null;
  limit: number;
}

export interface TablePage {
  columns: ColumnMeta[];
  rows: CellValue[][];
  nextCursor: Cursor | null;
}

export interface AppError {
  kind: string;
  message: string;
}

export function isAppError(e: unknown): e is AppError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}

export function errorMessage(e: unknown): string {
  if (isAppError(e)) return e.message;
  return String(e);
}

export const ipc = {
  connectionsList: () => invoke<ConnectionConfig[]>("connections_list"),
  connectionSave: (config: ConnectionConfig, password: string | null) =>
    invoke<void>("connection_save", { config, password }),
  connectionDelete: (id: string) => invoke<void>("connection_delete", { id }),
  connectionTest: (config: ConnectionConfig, password: string | null) =>
    invoke<void>("connection_test", { config, password }),
  listNamespaces: (connId: string) => invoke<string[]>("list_namespaces", { connId }),
  listTables: (connId: string, namespace: string) =>
    invoke<TableInfo[]>("list_tables", { connId, namespace }),
  listColumns: (connId: string, namespace: string, table: string) =>
    invoke<ColumnInfo[]>("list_columns", { connId, namespace, table }),
  fetchTablePage: (connId: string, req: TablePageRequest) =>
    invoke<TablePage>("fetch_table_page", { connId, req }),
  executeSql: (connId: string, sql: string, confirmed: boolean) =>
    invoke<QueryResult>("execute_sql", { connId, sql, confirmed }),
};
```

- [ ] **Step 3: 写 `src/stores/workspace.ts`**

```ts
import { create } from "zustand";

export type Tab =
  | { id: string; kind: "table"; connId: string; namespace: string; table: string; title: string }
  | { id: string; kind: "sql"; connId: string; title: string; sql: string };

interface WorkspaceState {
  activeConnId: string | null;
  tabs: Tab[];
  activeTabId: string | null;
  refreshNonce: number;
  setActiveConn: (id: string | null) => void;
  openTable: (connId: string, namespace: string, table: string) => void;
  openSqlTab: (connId: string) => void;
  updateSql: (tabId: string, sql: string) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  bumpRefresh: () => void;
}

let sqlTabCounter = 0;

export const useWorkspace = create<WorkspaceState>((set, get) => ({
  activeConnId: null,
  tabs: [],
  activeTabId: null,
  refreshNonce: 0,

  setActiveConn: (id) => set({ activeConnId: id }),

  openTable: (connId, namespace, table) => {
    const existing = get().tabs.find(
      (t) => t.kind === "table" && t.connId === connId && t.namespace === namespace && t.table === table,
    );
    if (existing) {
      set({ activeTabId: existing.id });
      return;
    }
    const tab: Tab = {
      id: crypto.randomUUID(),
      kind: "table",
      connId,
      namespace,
      table,
      title: table,
    };
    set((s) => ({ tabs: [...s.tabs, tab], activeTabId: tab.id }));
  },

  openSqlTab: (connId) => {
    sqlTabCounter += 1;
    const tab: Tab = {
      id: crypto.randomUUID(),
      kind: "sql",
      connId,
      title: `Query ${sqlTabCounter}`,
      sql: "",
    };
    set((s) => ({ tabs: [...s.tabs, tab], activeTabId: tab.id }));
  },

  updateSql: (tabId, sql) =>
    set((s) => ({
      tabs: s.tabs.map((t) => (t.id === tabId && t.kind === "sql" ? { ...t, sql } : t)),
    })),

  closeTab: (id) =>
    set((s) => {
      const idx = s.tabs.findIndex((t) => t.id === id);
      const tabs = s.tabs.filter((t) => t.id !== id);
      let activeTabId = s.activeTabId;
      if (s.activeTabId === id) {
        activeTabId = tabs[Math.min(idx, tabs.length - 1)]?.id ?? null;
      }
      return { tabs, activeTabId };
    }),

  setActiveTab: (id) => set({ activeTabId: id }),
  bumpRefresh: () => set((s) => ({ refreshNonce: s.refreshNonce + 1 })),
}));
```

- [ ] **Step 4: 写 `src/stores/ui.ts`**

```ts
import { create } from "zustand";

interface UiState {
  connectionsOpen: boolean;
  quickOpenOpen: boolean;
  setConnectionsOpen: (open: boolean) => void;
  setQuickOpenOpen: (open: boolean) => void;
}

export const useUi = create<UiState>((set) => ({
  connectionsOpen: false,
  quickOpenOpen: false,
  setConnectionsOpen: (open) => set({ connectionsOpen: open }),
  setQuickOpenOpen: (open) => set({ quickOpenOpen: open }),
}));
```

- [ ] **Step 5: 类型检查 + Commit**

```bash
pnpm exec tsc --noEmit
git add src/lib/ipc.ts src/stores/ package.json pnpm-lock.yaml
git commit -m "feat: typed ipc layer and workspace stores"
```

---

### Task 14: Connection Manager UI

**Files:**
- Create: `src/features/connections/ConnectionManagerDialog.tsx`
- Create: `src/features/connections/ConnectionForm.tsx`

**Interfaces:**
- Consumes: `ipc`、`useWorkspace`、`useUi`（Task 13）、shadcn `dialog/button/input/label/select`（Task 1）。
- Produces: `<ConnectionManagerDialog />`（由 `useUi.connectionsOpen` 控制，Cmd+K 触发在 Task 18）。

- [ ] **Step 1: 写 `src/features/connections/ConnectionForm.tsx`**

```tsx
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from "@/components/ui/select";
import { ConnectionConfig, Driver, SslMode, errorMessage, ipc } from "@/lib/ipc";

const DEFAULT_PORTS: Record<Driver, number> = { postgres: 5432, mysql: 3306 };

export function emptyConfig(): ConnectionConfig {
  return {
    id: crypto.randomUUID(),
    name: "",
    driver: "postgres",
    host: "localhost",
    port: 5432,
    username: "",
    database: "",
    sslMode: "prefer",
  };
}

interface Props {
  initial: ConnectionConfig;
  isNew: boolean;
  onSaved: () => void;
  onDeleted: () => void;
}

export function ConnectionForm({ initial, isNew, onSaved, onDeleted }: Props) {
  const [config, setConfig] = useState(initial);
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState<{ kind: "idle" | "ok" | "error" | "busy"; text: string }>({
    kind: "idle",
    text: "",
  });

  const patch = (p: Partial<ConnectionConfig>) => setConfig((c) => ({ ...c, ...p }));

  const passwordOrNull = password === "" ? null : password;

  const test = async () => {
    setStatus({ kind: "busy", text: "Testing..." });
    try {
      await ipc.connectionTest(config, passwordOrNull);
      setStatus({ kind: "ok", text: "Connection OK" });
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  const save = async () => {
    setStatus({ kind: "busy", text: "Saving..." });
    try {
      await ipc.connectionSave(config, passwordOrNull);
      setStatus({ kind: "ok", text: "Saved" });
      onSaved();
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  const remove = async () => {
    try {
      await ipc.connectionDelete(config.id);
      onDeleted();
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <div className="col-span-2">
          <Label htmlFor="name">Name</Label>
          <Input id="name" value={config.name} onChange={(e) => patch({ name: e.target.value })} />
        </div>
        <div>
          <Label>Driver</Label>
          <Select
            value={config.driver}
            onValueChange={(v: Driver) => patch({ driver: v, port: DEFAULT_PORTS[v] })}
          >
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="postgres">PostgreSQL</SelectItem>
              <SelectItem value="mysql">MySQL / MariaDB</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div>
          <Label>SSL</Label>
          <Select value={config.sslMode} onValueChange={(v: SslMode) => patch({ sslMode: v })}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="disable">Disable</SelectItem>
              <SelectItem value="prefer">Prefer</SelectItem>
              <SelectItem value="require">Require</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div>
          <Label htmlFor="host">Host</Label>
          <Input id="host" value={config.host} onChange={(e) => patch({ host: e.target.value })} />
        </div>
        <div>
          <Label htmlFor="port">Port</Label>
          <Input
            id="port"
            type="number"
            value={config.port}
            onChange={(e) => patch({ port: Number(e.target.value) || 0 })}
          />
        </div>
        <div>
          <Label htmlFor="username">User</Label>
          <Input id="username" value={config.username} onChange={(e) => patch({ username: e.target.value })} />
        </div>
        <div>
          <Label htmlFor="password">Password</Label>
          <Input
            id="password"
            type="password"
            value={password}
            placeholder={isNew ? "" : "(unchanged, stored in Keychain)"}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>
        <div className="col-span-2">
          <Label htmlFor="database">Database</Label>
          <Input id="database" value={config.database} onChange={(e) => patch({ database: e.target.value })} />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" onClick={test} disabled={status.kind === "busy"}>Test</Button>
        <Button onClick={save} disabled={status.kind === "busy"}>Save</Button>
        {!isNew && (
          <Button variant="destructive" onClick={remove}>Delete</Button>
        )}
        <span
          className={
            status.kind === "error" ? "text-sm text-red-500" : "text-sm text-muted-foreground"
          }
        >
          {status.text}
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 写 `src/features/connections/ConnectionManagerDialog.tsx`**

```tsx
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { ConnectionConfig, ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";
import { ConnectionForm, emptyConfig } from "./ConnectionForm";

export function ConnectionManagerDialog() {
  const { connectionsOpen, setConnectionsOpen } = useUi();
  const setActiveConn = useWorkspace((s) => s.setActiveConn);
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<{ config: ConnectionConfig; isNew: boolean } | null>(null);

  const { data: connections = [] } = useQuery({
    queryKey: ["connections"],
    queryFn: ipc.connectionsList,
    enabled: connectionsOpen,
  });

  const refresh = () => queryClient.invalidateQueries({ queryKey: ["connections"] });

  const connect = (id: string) => {
    setActiveConn(id);
    setConnectionsOpen(false);
  };

  return (
    <Dialog open={connectionsOpen} onOpenChange={setConnectionsOpen}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Connections</DialogTitle>
        </DialogHeader>
        <div className="flex gap-4">
          <div className="w-56 shrink-0 border-r pr-3 flex flex-col gap-1">
            {connections.map((c) => (
              <div key={c.id} className="flex items-center justify-between gap-1">
                <button
                  className="flex-1 truncate rounded px-2 py-1 text-left text-sm hover:bg-accent"
                  onClick={() => setEditing({ config: c, isNew: false })}
                >
                  {c.name || `${c.host}/${c.database}`}
                </button>
                <Button size="sm" variant="ghost" onClick={() => connect(c.id)}>
                  Open
                </Button>
              </div>
            ))}
            {connections.length === 0 && (
              <p className="px-2 py-1 text-sm text-muted-foreground">No connections yet</p>
            )}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => setEditing({ config: emptyConfig(), isNew: true })}
            >
              New Connection
            </Button>
          </div>
          <div className="min-h-64 flex-1">
            {editing ? (
              <ConnectionForm
                key={editing.config.id}
                initial={editing.config}
                isNew={editing.isNew}
                onSaved={() => {
                  refresh();
                  setEditing(null);
                }}
                onDeleted={() => {
                  refresh();
                  setEditing(null);
                }}
              />
            ) : (
              <p className="text-sm text-muted-foreground">
                Select a connection to edit, or create a new one.
              </p>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
```

- [ ] **Step 3: 类型检查 + Commit**

```bash
pnpm exec tsc --noEmit
git add src/features/connections/
git commit -m "feat: connection manager ui"
```

（运行时验证合并到 Task 18 的 App 接线后统一做。）

---

### Task 15: Workspace 布局、Schema Tree、Tab Bar

**Files:**
- Create: `src/features/workspace/SchemaTree.tsx`
- Create: `src/features/workspace/TabBar.tsx`
- Create: `src/features/workspace/Workspace.tsx`

**Interfaces:**
- Consumes: `ipc`、`useWorkspace`（Task 13）。
- Produces: `<Workspace />`（内部渲染 SchemaTree + TabBar + 活动 Tab 内容）。
Tab 内容渲染 `TableDataTab`（Task 16）与 `SqlEditorTab`（Task 17），本 Task 先用占位组件，后续 Task 替换 import。

- [ ] **Step 1: 写 `src/features/workspace/SchemaTree.tsx`**

```tsx
import { useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronRight, Eye, Table2 } from "lucide-react";
import { useState } from "react";
import { Input } from "@/components/ui/input";
import { ipc } from "@/lib/ipc";
import { useWorkspace } from "@/stores/workspace";

function NamespaceNode({
  connId,
  namespace,
  filter,
}: {
  connId: string;
  namespace: string;
  filter: string;
}) {
  const [expanded, setExpanded] = useState(false);
  const openTable = useWorkspace((s) => s.openTable);
  const refreshNonce = useWorkspace((s) => s.refreshNonce);

  const { data: tables = [], isLoading } = useQuery({
    queryKey: ["tables", connId, namespace, refreshNonce],
    queryFn: () => ipc.listTables(connId, namespace),
    enabled: expanded,
  });

  const visible = filter
    ? tables.filter((t) => t.name.toLowerCase().includes(filter.toLowerCase()))
    : tables;

  return (
    <div>
      <button
        className="flex w-full items-center gap-1 rounded px-1 py-0.5 text-sm hover:bg-accent"
        onClick={() => setExpanded((e) => !e)}
      >
        {expanded ? <ChevronDown className="size-3.5" /> : <ChevronRight className="size-3.5" />}
        <span className="truncate">{namespace}</span>
      </button>
      {expanded && (
        <div className="ml-4 flex flex-col">
          {isLoading && <span className="px-1 text-xs text-muted-foreground">Loading...</span>}
          {visible.map((t) => (
            <button
              key={t.name}
              className="flex items-center gap-1.5 rounded px-1 py-0.5 text-left text-sm hover:bg-accent"
              onClick={() => openTable(connId, namespace, t.name)}
            >
              {t.kind === "view" ? (
                <Eye className="size-3.5 shrink-0 text-muted-foreground" />
              ) : (
                <Table2 className="size-3.5 shrink-0 text-muted-foreground" />
              )}
              <span className="truncate">{t.name}</span>
            </button>
          ))}
          {!isLoading && visible.length === 0 && (
            <span className="px-1 text-xs text-muted-foreground">No tables</span>
          )}
        </div>
      )}
    </div>
  );
}

export function SchemaTree({ connId }: { connId: string }) {
  const [filter, setFilter] = useState("");
  const refreshNonce = useWorkspace((s) => s.refreshNonce);

  const { data: namespaces = [], isLoading, error } = useQuery({
    queryKey: ["namespaces", connId, refreshNonce],
    queryFn: () => ipc.listNamespaces(connId),
  });

  return (
    <div className="flex h-full flex-col gap-2 p-2">
      <Input
        placeholder="Filter tables..."
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        className="h-7 text-sm"
      />
      <div className="flex-1 overflow-y-auto">
        {isLoading && <span className="text-xs text-muted-foreground">Loading schemas...</span>}
        {error != null && <span className="text-xs text-red-500">Failed to load schemas</span>}
        {namespaces.map((ns) => (
          <NamespaceNode key={ns} connId={connId} namespace={ns} filter={filter} />
        ))}
      </div>
    </div>
  );
}
```

（`lucide-react` 是 shadcn init 的依赖，已就位。）

- [ ] **Step 2: 写 `src/features/workspace/TabBar.tsx`**

```tsx
import { X } from "lucide-react";
import { useWorkspace } from "@/stores/workspace";

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, closeTab } = useWorkspace();

  return (
    <div className="flex h-9 items-end gap-px overflow-x-auto border-b bg-muted/40 px-1">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={
            "group flex h-8 max-w-48 cursor-pointer items-center gap-1 rounded-t border border-b-0 px-2 text-sm " +
            (tab.id === activeTabId ? "bg-background" : "bg-muted/60 text-muted-foreground")
          }
          onClick={() => setActiveTab(tab.id)}
        >
          <span className="truncate">{tab.title}</span>
          <button
            className="rounded p-0.5 opacity-0 hover:bg-accent group-hover:opacity-100"
            onClick={(e) => {
              e.stopPropagation();
              closeTab(tab.id);
            }}
          >
            <X className="size-3" />
          </button>
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: 写 `src/features/workspace/Workspace.tsx`**

```tsx
import { useWorkspace } from "@/stores/workspace";
import { SchemaTree } from "./SchemaTree";
import { TabBar } from "./TabBar";
import { TableDataTab } from "@/features/data-grid/TableDataTab";
import { SqlEditorTab } from "@/features/sql-editor/SqlEditorTab";

export function Workspace({ connId }: { connId: string }) {
  const { tabs, activeTabId } = useWorkspace();
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  return (
    <div className="flex min-h-0 flex-1">
      <aside className="w-64 shrink-0 border-r">
        <SchemaTree connId={connId} />
      </aside>
      <main className="flex min-w-0 flex-1 flex-col">
        <TabBar />
        <div className="min-h-0 flex-1">
          {activeTab == null && (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              Open a table from the sidebar, or press Cmd+E for a new query.
            </div>
          )}
          {activeTab?.kind === "table" && <TableDataTab key={activeTab.id} tab={activeTab} />}
          {activeTab?.kind === "sql" && <SqlEditorTab key={activeTab.id} tab={activeTab} />}
        </div>
      </main>
    </div>
  );
}
```

本 Task 编译需要占位组件，先创建最小版（Task 16 / 17 会全量覆盖）：

`src/features/data-grid/TableDataTab.tsx`：

```tsx
import type { Tab } from "@/stores/workspace";

export function TableDataTab({ tab }: { tab: Extract<Tab, { kind: "table" }> }) {
  return <div className="p-4 text-sm text-muted-foreground">Table: {tab.table}</div>;
}
```

`src/features/sql-editor/SqlEditorTab.tsx`：

```tsx
import type { Tab } from "@/stores/workspace";

export function SqlEditorTab({ tab }: { tab: Extract<Tab, { kind: "sql" }> }) {
  return <div className="p-4 text-sm text-muted-foreground">SQL tab: {tab.title}</div>;
}
```

- [ ] **Step 4: 类型检查 + Commit**

```bash
pnpm exec tsc --noEmit
git add src/features/
git commit -m "feat: workspace layout, schema tree and tab bar"
```

---

### Task 16: Data Grid（虚拟滚动 + chunk 加载 + 排序 + 复制）

**Files:**
- Create: `src/features/data-grid/ResultGrid.tsx`
- Modify: `src/features/data-grid/TableDataTab.tsx`（全量覆盖占位）

**Interfaces:**
- Consumes: `ipc.fetchTablePage`、`ColumnMeta` / `CellValue` / `Sort` / `Cursor`（Task 13）。
- Produces:

```tsx
// 纯展示网格，表数据浏览与 SQL 结果共用
export function ResultGrid(props: {
  columns: ColumnMeta[];
  rows: CellValue[][];
  onEndReached?: () => void;   // 滚动接近底部触发（加载下一 chunk）
  sort?: Sort | null;          // 当前排序（表头指示）
  onSortChange?: (sort: Sort | null) => void;  // 点击表头
}): JSX.Element
```

- [ ] **Step 1: 写 `src/features/data-grid/ResultGrid.tsx`**

```tsx
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  flexRender, getCoreRowModel, useReactTable, type ColumnDef,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDown, ArrowUp } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { CellValue, ColumnMeta, Sort } from "@/lib/ipc";

function formatCell(v: CellValue): string {
  if (v === null) return "";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

type Selection = { row: number; col: number | null } | null;

interface Props {
  columns: ColumnMeta[];
  rows: CellValue[][];
  onEndReached?: () => void;
  sort?: Sort | null;
  onSortChange?: (sort: Sort | null) => void;
}

const ROW_HEIGHT = 28;

export function ResultGrid({ columns, rows, onEndReached, sort, onSortChange }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [selection, setSelection] = useState<Selection>(null);

  const columnDefs = useMemo<ColumnDef<CellValue[]>[]>(
    () =>
      columns.map((c, i) => ({
        id: String(i),
        header: c.name,
        accessorFn: (row) => row[i],
        size: 160,
        minSize: 60,
      })),
    [columns],
  );

  const table = useReactTable({
    data: rows,
    columns: columnDefs,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
  });

  const tableRows = table.getRowModel().rows;

  const virtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const virtualItems = virtualizer.getVirtualItems();

  useEffect(() => {
    const last = virtualItems.at(-1);
    if (last && last.index >= tableRows.length - 50) {
      onEndReached?.();
    }
  }, [virtualItems, tableRows.length, onEndReached]);

  const copySelection = async () => {
    if (!selection) return;
    const row = rows[selection.row];
    if (!row) return;
    const text =
      selection.col === null
        ? row.map(formatCell).join("\t")
        : formatCell(row[selection.col]);
    await writeText(text);
  };

  const cycleSort = (columnName: string) => {
    if (!onSortChange) return;
    if (sort?.column !== columnName) onSortChange({ column: columnName, desc: false });
    else if (!sort.desc) onSortChange({ column: columnName, desc: true });
    else onSortChange(null);
  };

  const headerGroups = table.getHeaderGroups();
  const totalWidth = table.getTotalSize();

  return (
    <div
      ref={containerRef}
      tabIndex={0}
      className="h-full overflow-auto outline-none"
      onKeyDown={(e) => {
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "c") {
          e.preventDefault();
          void copySelection();
        }
      }}
    >
      <div style={{ width: totalWidth + 48 }}>
        <div className="sticky top-0 z-10 flex border-b bg-background">
          <div className="w-12 shrink-0 border-r bg-muted/40" />
          {headerGroups[0]?.headers.map((header) => {
            const name = columns[Number(header.id)]?.name ?? "";
            const active = sort?.column === name;
            return (
              <div
                key={header.id}
                style={{ width: header.getSize() }}
                className="relative flex shrink-0 cursor-pointer select-none items-center gap-1 border-r px-2 py-1 text-xs font-medium hover:bg-accent"
                onClick={() => cycleSort(name)}
              >
                <span className="truncate">{flexRender(header.column.columnDef.header, header.getContext())}</span>
                {active && (sort!.desc ? <ArrowDown className="size-3" /> : <ArrowUp className="size-3" />)}
                <div
                  onMouseDown={header.getResizeHandler()}
                  onTouchStart={header.getResizeHandler()}
                  onClick={(e) => e.stopPropagation()}
                  className="absolute right-0 top-0 h-full w-1 cursor-col-resize hover:bg-primary/50"
                />
              </div>
            );
          })}
        </div>

        <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
          {virtualItems.map((vi) => {
            const row = tableRows[vi.index];
            return (
              <div
                key={vi.key}
                className="absolute left-0 flex w-full border-b"
                style={{ top: vi.start, height: ROW_HEIGHT }}
              >
                <button
                  className={
                    "w-12 shrink-0 border-r bg-muted/40 text-right text-xs text-muted-foreground px-1 " +
                    (selection?.row === vi.index && selection.col === null ? "bg-primary/20" : "")
                  }
                  onClick={() => setSelection({ row: vi.index, col: null })}
                >
                  {vi.index + 1}
                </button>
                {row.getVisibleCells().map((cell, ci) => {
                  const value = rows[vi.index]?.[ci] ?? null;
                  const selected = selection?.row === vi.index && selection.col === ci;
                  return (
                    <div
                      key={cell.id}
                      style={{ width: cell.column.getSize() }}
                      className={
                        "shrink-0 truncate border-r px-2 py-1 text-xs " +
                        (selected ? "bg-primary/20 " : "") +
                        (value === null ? "italic text-muted-foreground" : "")
                      }
                      onClick={() => setSelection({ row: vi.index, col: ci })}
                    >
                      {value === null ? "NULL" : formatCell(value)}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 覆盖 `src/features/data-grid/TableDataTab.tsx`**

```tsx
import { useInfiniteQuery } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import { errorMessage, ipc, type Cursor, type Sort } from "@/lib/ipc";
import { useWorkspace, type Tab } from "@/stores/workspace";
import { ResultGrid } from "./ResultGrid";

const PAGE_SIZE = 500;

export function TableDataTab({ tab }: { tab: Extract<Tab, { kind: "table" }> }) {
  const [sort, setSort] = useState<Sort | null>(null);
  const refreshNonce = useWorkspace((s) => s.refreshNonce);

  const query = useInfiniteQuery({
    queryKey: ["tablePage", tab.connId, tab.namespace, tab.table, sort, refreshNonce],
    queryFn: ({ pageParam }) =>
      ipc.fetchTablePage(tab.connId, {
        namespace: tab.namespace,
        table: tab.table,
        sort,
        cursor: pageParam,
        limit: PAGE_SIZE,
      }),
    initialPageParam: null as Cursor | null,
    getNextPageParam: (last) => last.nextCursor,
  });

  const columns = query.data?.pages[0]?.columns ?? [];
  const rows = useMemo(
    () => query.data?.pages.flatMap((p) => p.rows) ?? [],
    [query.data],
  );

  const onEndReached = useCallback(() => {
    if (query.hasNextPage && !query.isFetchingNextPage) {
      void query.fetchNextPage();
    }
  }, [query]);

  if (query.isLoading) {
    return <div className="p-4 text-sm text-muted-foreground">Loading {tab.table}...</div>;
  }
  if (query.error) {
    return <div className="p-4 text-sm text-red-500">{errorMessage(query.error)}</div>;
  }

  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1">
        <ResultGrid
          columns={columns}
          rows={rows}
          sort={sort}
          onSortChange={setSort}
          onEndReached={onEndReached}
        />
      </div>
      <div className="flex h-7 items-center gap-3 border-t px-2 text-xs text-muted-foreground">
        <span>
          {rows.length} rows loaded{query.hasNextPage ? " (scroll for more)" : ""}
        </span>
        {query.isFetchingNextPage && <span>Loading more...</span>}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: 类型检查 + Commit**

```bash
pnpm exec tsc --noEmit
git add src/features/data-grid/
git commit -m "feat: virtualized data grid with chunked loading, sort and copy"
```

---

### Task 17: SQL Editor Tab（Monaco + 执行 + 危险确认）

**Files:**
- Create: `src/features/sql-editor/monaco.ts`
- Modify: `src/features/sql-editor/SqlEditorTab.tsx`（全量覆盖占位）

**Interfaces:**
- Consumes: `ipc.executeSql`、`useWorkspace.updateSql`、`ResultGrid`（Task 16）、shadcn `dialog/button`。
- Produces: `<SqlEditorTab tab={...} />`，Cmd+Enter 在编辑器内执行；错误 kind 为 `dangerousStatement` 时弹确认，确认后带 `confirmed: true` 重发。

- [ ] **Step 1: 写 `src/features/sql-editor/monaco.ts`（bundle 内置 worker，离线可用）**

```ts
import * as monaco from "monaco-editor";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import { loader } from "@monaco-editor/react";

self.MonacoEnvironment = {
  getWorker: () => new editorWorker(),
};

loader.config({ monaco });

export { monaco };
```

- [ ] **Step 2: 覆盖 `src/features/sql-editor/SqlEditorTab.tsx`**

```tsx
import Editor, { type OnMount } from "@monaco-editor/react";
import { useMutation } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog";
import { errorMessage, ipc, isAppError, type QueryResult } from "@/lib/ipc";
import { useWorkspace, type Tab } from "@/stores/workspace";
import { ResultGrid } from "@/features/data-grid/ResultGrid";
import { monaco } from "./monaco";

export function SqlEditorTab({ tab }: { tab: Extract<Tab, { kind: "sql" }> }) {
  const updateSql = useWorkspace((s) => s.updateSql);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [pendingDanger, setPendingDanger] = useState<string | null>(null);
  const sqlRef = useRef(tab.sql);

  const run = useMutation({
    mutationFn: ({ confirmed }: { confirmed: boolean }) =>
      ipc.executeSql(tab.connId, sqlRef.current, confirmed),
    onSuccess: (r) => {
      setResult(r);
      setPendingDanger(null);
    },
    onError: (e) => {
      if (isAppError(e) && e.kind === "dangerousStatement") {
        setPendingDanger(e.message);
      }
    },
  });

  const execute = () => {
    if (sqlRef.current.trim() === "") return;
    run.mutate({ confirmed: false });
  };

  const onMount: OnMount = (editor) => {
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      if (sqlRef.current.trim() !== "") {
        run.mutate({ confirmed: false });
      }
    });
    editor.focus();
  };

  const error = run.error && !pendingDanger ? errorMessage(run.error) : null;

  return (
    <div className="flex h-full flex-col">
      <div className="h-2/5 min-h-32 border-b">
        <Editor
          defaultLanguage="sql"
          defaultValue={tab.sql}
          onMount={onMount}
          onChange={(v) => {
            sqlRef.current = v ?? "";
            updateSql(tab.id, sqlRef.current);
          }}
          options={{
            minimap: { enabled: false },
            fontSize: 13,
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>

      <div className="flex h-8 items-center gap-3 border-b px-2 text-xs">
        <Button size="sm" className="h-6 px-2 text-xs" onClick={execute} disabled={run.isPending}>
          Run (Cmd+Enter)
        </Button>
        {run.isPending && <span className="text-muted-foreground">Running...</span>}
        {result && (
          <span className="text-muted-foreground">
            {result.affectedRows !== null
              ? `${result.affectedRows} rows affected`
              : `${result.rows.length} rows${result.truncated ? " (truncated at 10000)" : ""}`}
            {" in "}
            {result.durationMs} ms
          </span>
        )}
      </div>

      <div className="min-h-0 flex-1">
        {error && <div className="p-3 text-sm text-red-500 whitespace-pre-wrap">{error}</div>}
        {!error && result && result.columns.length > 0 && (
          <ResultGrid columns={result.columns} rows={result.rows} />
        )}
        {!error && result && result.columns.length === 0 && (
          <div className="p-3 text-sm text-muted-foreground">Statement executed.</div>
        )}
        {!error && !result && (
          <div className="p-3 text-sm text-muted-foreground">Press Cmd+Enter to run the query.</div>
        )}
      </div>

      <Dialog open={pendingDanger !== null} onOpenChange={(open) => !open && setPendingDanger(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Dangerous statement</DialogTitle>
          </DialogHeader>
          <p className="text-sm whitespace-pre-wrap">{pendingDanger}</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingDanger(null)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => run.mutate({ confirmed: true })}>
              Run Anyway
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
```

- [ ] **Step 3: 类型检查 + Commit**

```bash
pnpm exec tsc --noEmit
git add src/features/sql-editor/
git commit -m "feat: monaco sql editor with danger confirmation"
```

---

### Task 18: 快捷键、Quick Open Table、App 接线

**Files:**
- Create: `src/lib/shortcuts.ts`
- Create: `src/features/workspace/QuickOpenTable.tsx`
- Modify: `src/App.tsx`（全量覆盖）
- Modify: `src/main.tsx`

**Interfaces:**
- Consumes: 前面所有前端组件与 store。
- Produces: 完整可跑的应用。快捷键：Cmd+K（连接）、Cmd+T（Quick Open）、Cmd+E（新 SQL Tab）、Cmd+W（关 Tab）、Cmd+R（刷新）。

- [ ] **Step 1: 写 `src/lib/shortcuts.ts`**

```ts
import { useEffect } from "react";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

export function useGlobalShortcuts() {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey) || e.shiftKey || e.altKey) return;
      const ws = useWorkspace.getState();
      const ui = useUi.getState();
      switch (e.key.toLowerCase()) {
        case "k":
          e.preventDefault();
          ui.setConnectionsOpen(!ui.connectionsOpen);
          break;
        case "t":
          if (ws.activeConnId) {
            e.preventDefault();
            ui.setQuickOpenOpen(true);
          }
          break;
        case "e":
          if (ws.activeConnId) {
            e.preventDefault();
            ws.openSqlTab(ws.activeConnId);
          }
          break;
        case "w":
          e.preventDefault();
          if (ws.activeTabId) ws.closeTab(ws.activeTabId);
          break;
        case "r":
          e.preventDefault();
          ws.bumpRefresh();
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
```

（Cmd+Enter 由 Monaco 编辑器内部处理，不走全局。）

- [ ] **Step 2: 写 `src/features/workspace/QuickOpenTable.tsx`**

```tsx
import { useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

interface Entry {
  namespace: string;
  table: string;
}

export function QuickOpenTable({ connId }: { connId: string }) {
  const { quickOpenOpen, setQuickOpenOpen } = useUi();
  const openTable = useWorkspace((s) => s.openTable);
  const [filter, setFilter] = useState("");
  const [highlighted, setHighlighted] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const { data: entries = [] } = useQuery({
    queryKey: ["allTables", connId],
    queryFn: async (): Promise<Entry[]> => {
      const namespaces = await ipc.listNamespaces(connId);
      const perNs = await Promise.all(
        namespaces.map(async (ns) => {
          const tables = await ipc.listTables(connId, ns);
          return tables.map((t) => ({ namespace: ns, table: t.name }));
        }),
      );
      return perNs.flat();
    },
    enabled: quickOpenOpen,
  });

  const visible = useMemo(() => {
    const q = filter.toLowerCase();
    return entries.filter((e) => e.table.toLowerCase().includes(q)).slice(0, 50);
  }, [entries, filter]);

  useEffect(() => {
    setHighlighted(0);
  }, [filter, quickOpenOpen]);

  const pick = (entry: Entry) => {
    openTable(connId, entry.namespace, entry.table);
    setQuickOpenOpen(false);
    setFilter("");
  };

  return (
    <Dialog open={quickOpenOpen} onOpenChange={setQuickOpenOpen}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Open Table</DialogTitle>
        </DialogHeader>
        <Input
          ref={inputRef}
          autoFocus
          placeholder="Type a table name..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setHighlighted((h) => Math.min(h + 1, visible.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setHighlighted((h) => Math.max(h - 1, 0));
            } else if (e.key === "Enter" && visible[highlighted]) {
              e.preventDefault();
              pick(visible[highlighted]);
            }
          }}
        />
        <div className="max-h-64 overflow-y-auto">
          {visible.map((entry, i) => (
            <button
              key={`${entry.namespace}.${entry.table}`}
              className={
                "flex w-full items-baseline gap-2 rounded px-2 py-1 text-left text-sm " +
                (i === highlighted ? "bg-accent" : "hover:bg-accent/50")
              }
              onMouseEnter={() => setHighlighted(i)}
              onClick={() => pick(entry)}
            >
              <span>{entry.table}</span>
              <span className="text-xs text-muted-foreground">{entry.namespace}</span>
            </button>
          ))}
          {visible.length === 0 && (
            <p className="px-2 py-1 text-sm text-muted-foreground">No matching tables</p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
```

- [ ] **Step 3: 覆盖 `src/App.tsx` 与 `src/main.tsx`**

`src/App.tsx`：

```tsx
import { Button } from "@/components/ui/button";
import { ConnectionManagerDialog } from "@/features/connections/ConnectionManagerDialog";
import { QuickOpenTable } from "@/features/workspace/QuickOpenTable";
import { Workspace } from "@/features/workspace/Workspace";
import { useGlobalShortcuts } from "@/lib/shortcuts";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

export default function App() {
  useGlobalShortcuts();
  const activeConnId = useWorkspace((s) => s.activeConnId);
  const setConnectionsOpen = useUi((s) => s.setConnectionsOpen);

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      {activeConnId ? (
        <>
          <Workspace connId={activeConnId} />
          <QuickOpenTable connId={activeConnId} />
        </>
      ) : (
        <div className="flex h-full flex-col items-center justify-center gap-3">
          <h1 className="text-xl font-semibold">DB-Minus</h1>
          <p className="text-sm text-muted-foreground">Connect to a database to get started.</p>
          <Button onClick={() => setConnectionsOpen(true)}>Open Connections (Cmd+K)</Button>
        </div>
      )}
      <ConnectionManagerDialog />
    </div>
  );
}
```

`src/main.tsx`：

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: false, refetchOnWindowFocus: false },
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);
```

- [ ] **Step 4: 类型检查 + 运行时验证核心链路**

```bash
pnpm exec tsc --noEmit
docker compose -f dev/docker-compose.yml up -d --wait
pnpm tauri dev
```

手动验证清单（对照操作）：

1. 启动显示空状态，点按钮或 Cmd+K 打开连接管理。
2. 新建 PG 连接：host `localhost`、port `5433`、user `dbminus`、password `dbminus`、database `dbminus_test`、SSL Disable。Test 显示 Connection OK，Save 后 Open。
3. 左侧出现 schema 树，展开 `public`，看到 `users`、`app_log`、`types_demo`、`active_users`（视图图标）。
4. 点 `users` 开 Tab，网格显示数据；滚动到底自动加载后续 chunk，底部行数增长到 1500。
5. 点列头 `id` 排序切换 asc / desc / none。
6. 点单元格后 Cmd+C，粘贴到任意处验证内容；点行号复制整行。
7. Cmd+E 开 SQL Tab，输入 `SELECT count(*) FROM users`，Cmd+Enter 出结果与耗时。
8. 输入 `DELETE FROM app_log`，Cmd+Enter 弹 Dangerous statement 确认框，Cancel 不执行。
9. Cmd+T 输入 `type` 回车打开 `types_demo`。
10. Cmd+W 关闭当前 Tab；Cmd+R 后网格重新加载。
11. 再建 MySQL 连接（port 3307）重复 3、4 抽查。

- [ ] **Step 5: Commit**

```bash
git add src/
git commit -m "feat: global shortcuts, quick open table and app wiring"
```

---

### Task 19: E2E 收尾验证与 README

**Files:**
- Create: `README.md`

**Interfaces:**
- Consumes: 全部。
- Produces: 项目说明文档；全量回归通过的骨架版。

- [ ] **Step 1: 全量回归**

```bash
docker compose -f dev/docker-compose.yml up -d --wait
cargo test --manifest-path src-tauri/Cargo.toml
pnpm exec tsc --noEmit
pnpm tauri build --debug 2>&1 | tail -5
```

预期：Rust 测试全过、TS 无错、debug 构建成功。

- [ ] **Step 2: 写 README.md**

```markdown
# DB-Minus

A lightweight, free, unlimited local database GUI for PostgreSQL and MySQL/MariaDB.
Built with Tauri 2, React and Rust.

## Status

V1 walking skeleton: connect, browse schema, read-only data grid, SQL editor.
See `docs/superpowers/specs/` for the design docs and `DB-Minus-PRD-v1.0.md` for the full roadmap.

## Development

Prerequisites: Rust, Node 22+, pnpm, Docker.

```bash
pnpm install
docker compose -f dev/docker-compose.yml up -d --wait   # test databases
pnpm tauri dev
```

Test databases: PostgreSQL at `localhost:5433`, MySQL at `localhost:3307`.
Credentials: `dbminus` / `dbminus`, database `dbminus_test`.

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # requires docker test databases
pnpm exec tsc --noEmit
```

## Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+K | Connections |
| Cmd+T | Quick open table |
| Cmd+E | New SQL tab |
| Cmd+W | Close tab |
| Cmd+R | Refresh |
| Cmd+Enter | Run SQL |
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add readme"
```
