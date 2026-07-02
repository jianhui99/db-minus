# DB-Minus 产品需求文档（PRD）

> Version: v1.0  
> Project: **DB-Minus**  
> Goal: 打造一款极轻量、免费、无限制的本地数据库 GUI，提供接近 TablePlus 的开发体验。

## 1. 产品定位

DB-Minus 专注于 PostgreSQL 和 MySQL/MariaDB 的日常开发体验，而不是覆盖所有数据库生态。

### 核心目标

- 极速启动（1 秒以内）
- 键盘优先
- 数据安全
- 无限 Tab
- 免费、无功能限制

---

## 2. 技术架构

### 前端

- Tauri 2
- React
- TypeScript
- shadcn/ui
- TanStack Table
- Monaco Editor
- React Aria Tree

### 后端

- Rust
- sqlx
- SSH Tunnel
- Connection Pool

---

## 3. 功能需求

### Milestone 1（MVP）

#### Connection Manager

- PostgreSQL
- MySQL / MariaDB
- SSH Tunnel
- SSL
- Connection Test
- Readonly Connection
- Connection Group
- Favorite Connection

#### Workspace

- 无限 Tab
- 双栏布局
- Schema Tree
- Table Filter

#### Data Grid

- Virtual Scroll
- Cursor Pagination
- Chunk Loading
- 排序
- 过滤
- Freeze Column
- Resize Column
- Copy Cell
- Copy Row
- Copy INSERT / UPDATE / DELETE SQL
- 编辑单元格
- 新增/删除数据
- Pending Changes
- Apply / Discard
- View SQL

#### SQL Editor

- Monaco Editor
- Syntax Highlight
- Auto Complete
- Cmd + Enter
- 多结果集
- SQL History
- SQL Formatter
- Explain Query
- Execution Time
- Affected Rows
- Export CSV
- Export JSON

---

## 4. 数据安全

所有修改默认不会立即提交。

所有危险操作必须确认：

- DROP DATABASE
- DROP TABLE
- TRUNCATE
- DELETE WITHOUT WHERE
- UPDATE WITHOUT WHERE

Readonly Connection 默认禁止：

- DELETE
- UPDATE
- DROP
- TRUNCATE

---

## 5. 快捷键

| 快捷键 | 功能 |
|---------|------|
| Cmd + K | 打开连接 |
| Cmd + T | Quick Open Table |
| Cmd + E | 新建 SQL |
| Cmd + W | 关闭 Tab |
| Cmd + R | 刷新 |
| Cmd + Enter | 执行 SQL |
| Cmd + S | Apply Changes |
| Cmd + Shift + P | Command Palette |

---

## 6. 非功能需求

- 冷启动 < 1 秒
- 空载内存 < 100MB
- 查询 100k+ 数据不卡顿
- Worker Thread + Streaming Result
- 自动恢复 Workspace

---

## 7. Roadmap

### V1

- PostgreSQL
- MySQL
- SQL Editor
- Data Grid
- Connection Manager

### V2

- SQLite
- Docker / Podman / OrbStack / Colima 自动发现
- Table Designer
- Export Wizard

### V3

- AI Assistant（Ollama / OpenAI）
- Plugin System
- Database Diff

---

## 8. 产品原则

1. Speed First
2. Keyboard First
3. Safe by Default
4. Free Forever
5. Unlimited Tabs
