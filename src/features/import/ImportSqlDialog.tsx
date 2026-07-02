import { open } from "@tauri-apps/plugin-dialog";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog";
import { errorMessage, ipc, isAppError, type ImportResult } from "@/lib/ipc";
import { useUi } from "@/stores/ui";
import { useWorkspace } from "@/stores/workspace";

export function ImportSqlDialog({ connId }: { connId: string }) {
  const { importOpen, setImportOpen } = useUi();
  const [pickedPath, setPickedPath] = useState<string | null>(null);
  const [pendingDanger, setPendingDanger] = useState<string | null>(null);
  const [result, setResult] = useState<ImportResult | null>(null);

  const run = useMutation({
    mutationFn: ({ confirmed }: { confirmed: boolean }) =>
      ipc.importSqlFile(connId, pickedPath!, confirmed),
    onSuccess: (r) => {
      setResult(r);
      setPendingDanger(null);
      useWorkspace.getState().bumpRefresh();
    },
    onError: (e) => {
      if (isAppError(e) && e.kind === "dangerousStatement") {
        setPendingDanger(e.message);
      }
    },
  });

  const chooseFile = async () => {
    const picked = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "SQL", extensions: ["sql"] }],
    });
    if (typeof picked === "string") {
      setPickedPath(picked);
      setResult(null);
    }
  };

  const runImport = () => {
    if (!pickedPath) return;
    run.mutate({ confirmed: false });
  };

  const reset = () => {
    setPickedPath(null);
    setPendingDanger(null);
    setResult(null);
  };

  const error = run.error && !pendingDanger ? errorMessage(run.error) : null;
  const fileName = pickedPath?.split(/[\\/]/).pop() ?? null;

  return (
    <>
      <Dialog
        open={importOpen}
        onOpenChange={(open) => {
          setImportOpen(open);
          if (!open) reset();
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Import SQL File</DialogTitle>
          </DialogHeader>

          <div className="flex flex-col gap-3">
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm" onClick={chooseFile}>
                Choose File...
              </Button>
              <span className="truncate text-sm text-muted-foreground">
                {fileName ?? "No file selected"}
              </span>
            </div>

            <div className="flex items-center gap-3">
              <Button size="sm" onClick={runImport} disabled={!pickedPath || run.isPending}>
                Run Import
              </Button>
              {run.isPending && <span className="text-sm text-muted-foreground">Importing...</span>}
            </div>

            {error && <div className="text-sm text-red-500 whitespace-pre-wrap">{error}</div>}

            {!error && result && (
              <div className="text-sm">
                <p className={result.failedStatement ? "text-red-500" : "text-muted-foreground"}>
                  {result.executedStatements} / {result.totalStatements} statements executed in{" "}
                  {result.durationMs} ms.
                </p>
                {result.failedStatement && (
                  <>
                    <p className="text-red-500">
                      Statement #{result.failedStatement.index} failed: {result.failedStatement.message}
                    </p>
                    <p className="text-muted-foreground">
                      Everything before it stayed committed; nothing after it ran.
                    </p>
                  </>
                )}
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={pendingDanger !== null} onOpenChange={(open) => !open && setPendingDanger(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Dangerous statements found</DialogTitle>
          </DialogHeader>
          <p className="text-sm whitespace-pre-wrap">{pendingDanger}</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingDanger(null)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={() => run.mutate({ confirmed: true })}>
              Run Anyway
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
