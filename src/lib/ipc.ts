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

export interface FailedStatement {
  index: number;
  sql: string;
  message: string;
}

export interface ImportResult {
  totalStatements: number;
  executedStatements: number;
  durationMs: number;
  failedStatement: FailedStatement | null;
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
  importSqlFile: (connId: string, path: string, confirmed: boolean) =>
    invoke<ImportResult>("import_sql_file", { connId, path, confirmed }),
};
