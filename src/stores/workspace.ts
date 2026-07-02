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
