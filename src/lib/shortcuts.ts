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
        case "i":
          if (ws.activeConnId) {
            e.preventDefault();
            ui.setImportOpen(true);
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
