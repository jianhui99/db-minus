import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { ConnectionConfig, errorMessage, ipc } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";
import { ConnectionForm, emptyConfig } from "./ConnectionForm";

export function ConnectionManagerDialog() {
  const { connectionsOpen, setConnectionsOpen } = useUi();
  const setActiveConn = useWorkspace((s) => s.setActiveConn);
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<{ config: ConnectionConfig; isNew: boolean } | null>(null);
  const [deleting, setDeleting] = useState<{ id: string; name: string } | null>(null);
  const [deleteError, setDeleteError] = useState<string>("");

  const { data: connections = [] } = useQuery({
    queryKey: ["connections"],
    queryFn: ipc.connectionsList,
    enabled: connectionsOpen,
  });

  const refresh = () => queryClient.invalidateQueries({ queryKey: ["connections"] });

  const connect = (id: string) => {
    setActiveConn(id);
    setConnectionsOpen(false);
  };

  const handleDelete = async () => {
    if (!deleting) return;
    try {
      await ipc.connectionDelete(deleting.id);
      refresh();
      setDeleting(null);
      setDeleteError("");
    } catch (e) {
      setDeleteError(errorMessage(e));
    }
  };

  return (
    <Dialog open={connectionsOpen} onOpenChange={setConnectionsOpen}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Connections</DialogTitle>
        </DialogHeader>
        <div className="flex gap-4">
          <div className="w-56 shrink-0 border-r pr-3 flex flex-col gap-1">
            {connections.map((c) => (
              <div key={c.id} className="flex items-center justify-between gap-1">
                <button
                  className="flex-1 truncate rounded px-2 py-1 text-left text-sm hover:bg-accent"
                  onClick={() => setEditing({ config: c, isNew: false })}
                >
                  {c.name || `${c.host}/${c.database}`}
                </button>
                <Button size="sm" variant="ghost" onClick={() => connect(c.id)}>
                  Open
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setDeleting({ id: c.id, name: c.name || `${c.host}/${c.database}` })}
                >
                  <Trash2 className="size-4 text-destructive" />
                </Button>
              </div>
            ))}
            {connections.length === 0 && (
              <p className="px-2 py-1 text-sm text-muted-foreground">No connections yet</p>
            )}
            <Button
              variant="outline"
              size="sm"
              className="mt-2"
              onClick={() => setEditing({ config: emptyConfig(), isNew: true })}
            >
              New Connection
            </Button>
          </div>
          <div className="min-h-64 flex-1">
            {editing ? (
              <ConnectionForm
                key={editing.config.id}
                initial={editing.config}
                isNew={editing.isNew}
                onSaved={() => {
                  refresh();
                  setEditing(null);
                }}
                onDeleted={() => {
                  refresh();
                  setEditing(null);
                }}
              />
            ) : (
              <p className="text-sm text-muted-foreground">
                Select a connection to edit, or create a new one.
              </p>
            )}
          </div>
        </div>

        <Dialog open={deleting !== null} onOpenChange={(open) => !open && setDeleting(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Delete connection?</DialogTitle>
            </DialogHeader>
            <p className="text-sm text-muted-foreground">
              This will delete "{deleting?.name}". This cannot be undone.
            </p>
            {deleteError && <p className="text-sm text-red-500">{deleteError}</p>}
            <DialogFooter>
              <Button variant="outline" onClick={() => setDeleting(null)}>
                Cancel
              </Button>
              <Button variant="destructive" onClick={handleDelete}>
                Delete
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </DialogContent>
    </Dialog>
  );
}
