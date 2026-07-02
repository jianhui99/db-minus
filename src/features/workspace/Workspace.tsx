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
