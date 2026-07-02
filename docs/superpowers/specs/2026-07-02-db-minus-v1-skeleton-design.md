# DB-Minus V1 骨架设计文档

> Date: 2026-07-02
> Status: Approved
> Source: DB-Minus-PRD-v1.0.md (Milestone 1)

## 1. 目标

打通 DB-Minus 的核心链路：连接 PostgreSQL / MySQL / MariaDB，浏览 Schema 与表数据，执行 SQL 并查看结果。
本版本是可用骨架（walking skeleton），优先保证链路完整和架构正确，功能深度在后续迭代补齐。

## 2. 范围

### 包含（V1 骨架）

**Connection Manager（基础版）**

- 连接的新建、编辑、删除、列表。
- 支持 PostgreSQL 与 MySQL / MariaDB。
- 连接测试（Connection Test）。
- SSL 基础参数（mode / 开关级别，不含自定义证书链 UI）。
- 连接配置持久化到 app config 目录的 JSON 文件。
- 密码存入 macOS Keychain（`keyring` crate），配置文件中不落明文密码。

**Workspace**

- 双栏布局：左侧 Schema Tree，右侧 Tab 区。
- Schema Tree 层级：连接 → 数据库/Schema → 表。
- 表名过滤输入框（Table Filter）。
- 无限 Tab：表数据 Tab 与 SQL 编辑器 Tab 混排，可关闭、可切换。

**Data Grid（只读）**

- TanStack Table + 虚拟滚动。
- Cursor（keyset）分页，按 chunk 加载（每批 500 行），滚动到底自动加载下一批。
- 服务端排序（点击列头生成 ORDER BY）。
- 列宽调整（Resize Column）。
- 复制单元格、复制整行。

**SQL Editor**

- Monaco Editor，SQL 语法高亮。
- Cmd + Enter 执行当前语句。
- 显示执行耗时（Execution Time）与影响行数（Affected Rows）。
- 单结果集展示（复用 Data Grid 组件）。

**数据安全**

- 危险 SQL 执行前确认弹窗：DROP DATABASE、DROP TABLE、TRUNCATE、无 WHERE 的 DELETE、无 WHERE 的 UPDATE。
- 预检基于 SQL 语句的轻量解析（关键字级别），不追求完备的语法分析。

**快捷键**

| 快捷键 | 功能 |
|---------|------|
| Cmd + K | 打开连接选择 |
| Cmd + T | Quick Open Table |
| Cmd + E | 新建 SQL Tab |
| Cmd + W | 关闭当前 Tab |
| Cmd + R | 刷新当前 Tab |
| Cmd + Enter | 执行 SQL |

### 不包含（推迟到迭代 2+）

- SSH Tunnel、连接分组、收藏、Readonly Connection。
- Data Grid 编辑、新增/删除行、Pending Changes、Apply / Discard、View SQL、复制 INSERT / UPDATE / DELETE SQL、过滤器、冻结列。
- 多结果集、SQL History、SQL Formatter、Explain Query、Export CSV / JSON。
- Auto Complete（schema 感知补全）、Command Palette（Cmd + Shift + P）。
- Workspace 自动恢复。
- 流式结果传输（Tauri Channel）。

**决策说明**：Data Grid 编辑整体推迟，因为没有 Pending Changes 的直接写库违背 PRD 的 Safe by Default 原则。
编辑与 Pending Changes 在迭代 2 作为一个整体交付。

## 3. 技术架构

### 总览

- **壳与前端**：Tauri 2 + React + TypeScript + Vite + Tailwind CSS + shadcn/ui。
- **表格**：TanStack Table + TanStack Virtual。
- **编辑器**：Monaco Editor。
- **后端**：Rust + tokio + sqlx（`postgres`、`mysql` features）。
- **IPC**：标准 Tauri `invoke` command，JSON 序列化。
大结果集按 chunk 分页拉取，本版本不做流式传输。

### 多数据库分发

不使用 sqlx 的 `Any` 驱动（类型信息受限），改用手动枚举分发：

```rust
enum DbPool {
    Postgres(sqlx::PgPool),
    MySql(sqlx::MySqlPool),
}
```

MariaDB 走 MySQL 驱动。
方言差异（引号、元数据查询、keyset 分页 SQL）封装在 Rust 侧的 dialect 模块，前端不感知。

### 连接池管理

Tauri managed state 持有 `RwLock<HashMap<ConnectionId, DbPool>>`。
连接在首次使用时惰性建池，断开或删除连接时销毁。
查询在 tokio 线程池执行，不阻塞 UI 线程，满足 PRD 的 Worker Thread 非功能需求。

### Rust 侧模块划分

- `connection`：连接配置的 CRUD 与持久化、Keychain 读写、连接测试、池生命周期。
- `dialect`：PG / MySQL 方言差异（标识符引用、系统目录查询、分页 SQL 生成）。
- `schema`：元数据查询（数据库、schema、表、列列表）。
- `query`：SQL 执行、结果集序列化（列定义 + 行数据）、耗时与影响行数统计。
- `safety`：危险 SQL 预检。
- `commands`：Tauri command 层，只做参数校验与模块调用，不含业务逻辑。

### 前端模块划分

- `features/connections`：连接管理 UI（列表、表单、测试）。
- `features/workspace`：布局、Tab 管理、Schema Tree。
- `features/data-grid`：表格组件（浏览表数据与 SQL 结果集共用）。
- `features/sql-editor`：Monaco 封装、执行触发、结果展示。
- `lib/ipc`：Tauri invoke 的类型安全封装，与 Rust command 一一对应。
- 状态管理用 Zustand（Tab 状态、当前连接），服务端数据用 TanStack Query 管理缓存与加载态。

### 数据流（以浏览表数据为例）

1. 用户在 Schema Tree 点击表，前端开新 Tab。
2. Tab 挂载后 invoke `fetch_table_page(conn_id, table, cursor, sort)`。
3. Rust 按方言生成 keyset 分页 SQL，从池中取连接执行。
4. 返回 `{ columns, rows, next_cursor }`，前端 append 到 Grid。
5. 虚拟滚动接近底部时携带 `next_cursor` 拉下一批。

无主键的表退化为 LIMIT / OFFSET 分页。

### 错误处理

- Rust 侧统一 `AppError` 枚举（连接失败、查询错误、超时、Keychain 错误），经 `serde` 序列化为结构化错误传给前端。
- 前端按错误类别展示：连接类错误弹 toast 并标记连接状态，SQL 错误内联展示在结果区（含数据库原始错误信息）。
- 连接测试与查询设置超时（默认 10s / 30s），避免 UI 无限等待。

## 4. 测试策略

- **E2E 环境**：Docker Compose 起 `postgres:17` 与 `mysql:8`，带种子数据（覆盖常见类型、含/不含主键的表、大表）。
- **Rust 集成测试**：`connection`、`schema`、`query`、`dialect` 模块连真实容器测试。
- **Rust 单元测试**：`safety` 危险 SQL 预检、方言 SQL 生成等纯逻辑。
- **前端**：核心链路（建连接 → 浏览表 → 执行 SQL）通过 `tauri dev` 真机验证。
组件级自动化测试本版本不铺开。

## 5. 非功能需求对齐

| PRD 要求 | 骨架版做法 |
|-----------|-----------|
| 冷启动 < 1 秒 | Tauri 原生壳，前端按需加载 Monaco（懒加载 chunk） |
| 空载内存 < 100MB | 不预建连接池，惰性初始化 |
| 100k+ 数据不卡顿 | 虚拟滚动 + chunk 分页，DOM 只渲染可视行 |
| Worker Thread | 查询在 tokio 线程池执行 |
| Streaming Result | 本版本用 chunk 分页替代，流式推迟 |

## 6. 迭代路线

- **迭代 2**：编辑 + Pending Changes + Apply / Discard + View SQL、SSH Tunnel、Readonly Connection、过滤器、Export。
- **迭代 3**：SQL History、Formatter、Explain、多结果集、schema 感知补全、Command Palette、Workspace 恢复。
