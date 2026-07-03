import { useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronRight, Eye, Table2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

function NamespaceNode({
  connId,
  namespace,
  filter,
  isFirst,
}: {
  connId: string;
  namespace: string;
  filter: string;
  isFirst: boolean;
}) {
  const [expanded, setExpanded] = useState(isFirst);
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
      <div className="flex items-center gap-2">
        <Input
          placeholder="Filter tables..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="h-7 flex-1 text-sm"
        />
        <Button
          size="sm"
          variant="outline"
          className="h-7 px-2 text-xs"
          onClick={() => useUi.getState().setImportOpen(true)}
          title="Import SQL file (Cmd+I)"
        >
          Import
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto">
        {isLoading && <span className="text-xs text-muted-foreground">Loading schemas...</span>}
        {error != null && <span className="text-xs text-red-500">Failed to load schemas</span>}
        {namespaces.map((ns, i) => (
          <NamespaceNode key={ns} connId={connId} namespace={ns} filter={filter} isFirst={i === 0} />
        ))}
      </div>
    </div>
  );
}
