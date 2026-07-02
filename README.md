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
