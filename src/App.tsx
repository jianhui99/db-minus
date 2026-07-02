import { Button } from "@/components/ui/button";
import { ConnectionManagerDialog } from "@/features/connections/ConnectionManagerDialog";
import { ImportSqlDialog } from "@/features/import/ImportSqlDialog";
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
          <ImportSqlDialog connId={activeConnId} />
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
