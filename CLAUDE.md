# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

DB-Minus: a free, local-first desktop GUI for PostgreSQL and MySQL/MariaDB, built on Tauri 2 (Rust backend + React/TS frontend). Currently at the "V1 walking skeleton" stage: connect, browse schema, read-only paginated data grid, SQL editor. See `docs/superpowers/specs/2026-07-02-db-minus-v1-skeleton-design.md` for the design doc and `docs/superpowers/plans/2026-07-02-db-minus-v1-skeleton.md` for the task-by-task implementation plan (useful for understanding *why* things are shaped the way they are). `DB-Minus-PRD-v1.0.md` has the full product roadmap beyond V1.

## Commands

```bash
pnpm install
docker compose -f dev/docker-compose.yml up -d --wait   # test databases (required for Rust integration tests)
pnpm tauri dev                                            # run the app (live reload)
pnpm exec tsc --noEmit                                    # typecheck frontend (no separate lint step configured)
cargo test --manifest-path src-tauri/Cargo.toml           # all Rust tests (unit + integration)
cargo test --manifest-path src-tauri/Cargo.toml --test import_test   # one integration test file
cargo test --manifest-path src-tauri/Cargo.toml dialect   # unit tests matching a name, any file
pnpm build                                                # production build (backend + frontend)
pnpm tauri build --debug                                  # debug bundle sanity check
```

Test databases: PostgreSQL on `localhost:5433`, MySQL on `localhost:3307`. Credentials `dbminus`/`dbminus`, database `dbminus_test`. Rust integration tests (anything in `src-tauri/tests/`) hit these live containers directly — they will fail or hang without `docker compose up` first. The `connection::secret` Keychain round-trip test is `#[ignore]`d because it touches the real macOS Keychain; run explicitly with `cargo test -- --ignored secret`.

## Architecture

### Backend (`src-tauri/src/`), one module per concern

- `connection/config.rs` — `ConnectionConfig`/`Driver`/`SslMode` model plus `ConfigStore`, a flat JSON file (`connections.json` in the Tauri app-config dir) with list/save(upsert)/delete. No passwords in this file.
- `connection/secret.rs` — password storage via OS Keychain (`keyring` crate), keyed by connection id.
- `connection/pool.rs` — `DbPool` is a hand-rolled enum (`Postgres(PgPool) | MySql(MySqlPool))`; there is no trait-object abstraction over sqlx drivers, so most query code pattern-matches on this enum explicitly. `PoolManager` caches one pool per connection id.
- `dialect.rs` — the only place that knows how to quote identifiers, build connection URLs, and produce bind placeholders (`$1` vs `?`) per driver. Any new SQL-building code should go through this rather than hand-formatting strings.
- `safety.rs` — pre-execution scan for dangerous statements (DROP/TRUNCATE/DELETE or UPDATE without WHERE). Two functions: `analyze(sql)` scans and returns warnings (does not block), and `split_statements(sql)` splits on top-level `;` while preserving original text (used by import loop). Both use a conservative character-level state machine that tracks quotes and comments to avoid false positives inside string literals. `danger_label(kind)` is a helper for error messages.
- `schema.rs` — `information_schema` queries for namespaces/tables/columns per driver, plus `integer_primary_key` (single-column integer PK detection, used to decide keyset vs offset pagination).
- `query.rs` — two responsibilities: (1) typed row→`serde_json::Value` decoding shared by both raw SQL execution and table paging (UUID/NUMERIC/JSONB/BYTEA etc. each have explicit conversion rules — see the `try_decode!` macro and `pg_value_typed`/`mysql_value_typed`), and (2) `fetch_table_page`, which picks keyset pagination (`WHERE pk > cursor`) when the table has a single-column integer PK and no custom sort, else falls back to LIMIT/OFFSET.
- `import.rs` — `run_import(pool, script)` splits a `.sql` script via `safety::split_statements`, then loops calling `query::execute_sql` on each statement sequentially with **no transaction wrapper**. Stops at the first failure; earlier statements remain committed (intentional, matching MySQL DDL auto-commit behavior). Returns `ImportResult` with counts, duration, and the first failed statement (if any) as a reportable outcome, not an error.
- `commands.rs` — the only file with `#[tauri::command]` functions; thin wrappers that resolve a `DbPool` from `AppState` and delegate to the modules above. Each dangerous-operation command takes a `confirmed: bool` flag to gate behind user confirmation. The `summarize_dangers` helper groups safety warnings by kind for a single upfront confirmation message (prevents multiple confirmation dialogs for one script).
- `error.rs` — single `AppError` enum, serialized as `{ kind, message }` (tagged enum, camelCase) so the frontend can pattern-match on `kind` (e.g. `"dangerousStatement"` triggers a confirm dialog — see `SqlEditorTab.tsx`).

All IPC-facing Rust types use `#[serde(rename_all = "camelCase")]`; frontend TS types in `src/lib/ipc.ts` mirror them field-for-field. When changing a Rust command's signature or return type, update `src/lib/ipc.ts` in the same change.

### Frontend (`src/`)

- `lib/ipc.ts` — the single typed boundary to the Rust backend (wraps `@tauri-apps/api` `invoke`). All IPC types (`ConnectionConfig`, `TablePageRequest`, `QueryResult`, `ImportResult`, etc.) and the `ipc` call object live here. `isAppError`/`errorMessage` unwrap the `AppError` shape from the Rust side. Mirror TS types to Rust exactly (both use camelCase for IPC fields).
- `stores/workspace.ts` (Zustand) — the active connection, open tabs (`{ kind: "table" | "sql" }`), active tab id, and a `refreshNonce` counter that TanStack Query keys include to force refetches on Cmd+R.
- `stores/ui.ts` (Zustand) — transient dialog open/closed state (Connections manager, Quick Open, Import SQL).
- `features/connections/` — Connection Manager dialog + form (test/save/delete a connection).
- `features/workspace/` — `Workspace` (schema tree + tab bar + active tab content), `SchemaTree` (lazy-loads tables per namespace via TanStack Query, keyed off `refreshNonce`), `TabBar`, `QuickOpenTable` (fuzzy table search across all namespaces, Cmd+T).
- `features/data-grid/ResultGrid.tsx` — shared virtualized grid (TanStack Table + TanStack Virtual) used by both table browsing and SQL results. Column-resizable, click-to-select cell/row with Cmd+C copy via `@tauri-apps/plugin-clipboard-manager`, click-header to cycle sort asc/desc/none.
- `features/data-grid/TableDataTab.tsx` — drives `ResultGrid` off `useInfiniteQuery` + `ipc.fetchTablePage`, 500 rows per chunk, loads more near scroll bottom.
- `features/sql-editor/` — Monaco editor (`monaco.ts` wires the bundled web worker so it works fully offline) + `SqlEditorTab`. Cmd+Enter executes with `confirmed: false`; if the backend returns `AppError.kind === "dangerousStatement"`, a confirm dialog offers "Run Anyway" which resends with `confirmed: true`.
- `features/import/ImportSqlDialog.tsx` — file picker dialog for importing `.sql` scripts. Picks a file via native `@tauri-apps/plugin-dialog`, scans for dangers, shows a single grouped confirmation if found, then runs the import and displays a summary (count, duration, any failed statement detail). On success, calls `bumpRefresh()` to update the schema tree.
- `lib/shortcuts.ts` — the one global `keydown` listener (Cmd+K connections, Cmd+T quick open, Cmd+E new SQL tab, Cmd+I import SQL file, Cmd+W close tab, Cmd+R refresh). Only active when connected; Cmd+Enter for running SQL is bound inside the Monaco editor instead, not here.

### Cross-cutting conventions

- Data Grid pages load 500 rows at a time; raw SQL Editor results cap at 10,000 rows (`query::MAX_RESULT_ROWS`) and set `truncated: true` beyond that.
- Connect timeout is 10s, query timeout is 30s (`tokio::time::timeout` wraps both in `pool.rs`/`query.rs`).
- Keyset pagination only applies when a table has exactly one integer-typed primary key column *and* no custom sort is requested; everything else (composite/non-integer PKs, no PK, custom sort) uses LIMIT/OFFSET. This is a deliberate skeleton-stage simplification (binding arbitrary cursor value types is out of scope for V1) — see Global Constraints in the plan doc.
- UI copy is English-only; Rust/doc comments in this codebase are Chinese (matches the design/plan docs) — match whichever convention the file you're editing already uses.
- No `.at()`/ES2022 array features issue: `tsconfig.json` targets ES2022 specifically because TanStack Virtual's virtualizer output is consumed with `Array.prototype.at`.
- shadcn's `DialogContent` default className includes `sm:max-w-sm`. Overriding width requires passing an `sm:`-prefixed class (e.g. `sm:max-w-2xl`) — a bare `max-w-2xl` will lose to the `sm:` variant at ≥640px viewports (tailwind-merge doesn't dedupe across differing modifiers).
