import { useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

interface Entry {
  namespace: string;
  table: string;
}

export function QuickOpenTable({ connId }: { connId: string }) {
  const { quickOpenOpen, setQuickOpenOpen } = useUi();
  const openTable = useWorkspace((s) => s.openTable);
  const [filter, setFilter] = useState("");
  const [highlighted, setHighlighted] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const { data: entries = [] } = useQuery({
    queryKey: ["allTables", connId],
    queryFn: async (): Promise<Entry[]> => {
      const namespaces = await ipc.listNamespaces(connId);
      const perNs = await Promise.all(
        namespaces.map(async (ns) => {
          const tables = await ipc.listTables(connId, ns);
          return tables.map((t) => ({ namespace: ns, table: t.name }));
        }),
      );
      return perNs.flat();
    },
    enabled: quickOpenOpen,
  });

  const visible = useMemo(() => {
    const q = filter.toLowerCase();
    return entries.filter((e) => e.table.toLowerCase().includes(q)).slice(0, 50);
  }, [entries, filter]);

  useEffect(() => {
    setHighlighted(0);
  }, [filter, quickOpenOpen]);

  const pick = (entry: Entry) => {
    openTable(connId, entry.namespace, entry.table);
    setQuickOpenOpen(false);
    setFilter("");
  };

  return (
    <Dialog open={quickOpenOpen} onOpenChange={setQuickOpenOpen}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Open Table</DialogTitle>
        </DialogHeader>
        <Input
          ref={inputRef}
          autoFocus
          placeholder="Type a table name..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setHighlighted((h) => Math.min(h + 1, visible.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setHighlighted((h) => Math.max(h - 1, 0));
            } else if (e.key === "Enter" && visible[highlighted]) {
              e.preventDefault();
              pick(visible[highlighted]);
            }
          }}
        />
        <div className="max-h-64 overflow-y-auto">
          {visible.map((entry, i) => (
            <button
              key={`${entry.namespace}.${entry.table}`}
              className={
                "flex w-full items-baseline gap-2 rounded px-2 py-1 text-left text-sm " +
                (i === highlighted ? "bg-accent" : "hover:bg-accent/50")
              }
              onMouseEnter={() => setHighlighted(i)}
              onClick={() => pick(entry)}
            >
              <span>{entry.table}</span>
              <span className="text-xs text-muted-foreground">{entry.namespace}</span>
            </button>
          ))}
          {visible.length === 0 && (
            <p className="px-2 py-1 text-sm text-muted-foreground">No matching tables</p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
