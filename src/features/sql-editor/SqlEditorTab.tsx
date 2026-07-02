import Editor, { type OnMount } from "@monaco-editor/react";
import { useMutation } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog";
import { errorMessage, ipc, isAppError, type QueryResult } from "@/lib/ipc";
import { useWorkspace, type Tab } from "@/stores/workspace";
import { ResultGrid } from "@/features/data-grid/ResultGrid";
import { monaco } from "./monaco";

export function SqlEditorTab({ tab }: { tab: Extract<Tab, { kind: "sql" }> }) {
  const updateSql = useWorkspace((s) => s.updateSql);
  const [result, setResult] = useState<QueryResult | null>(null);
  const [pendingDanger, setPendingDanger] = useState<string | null>(null);
  const sqlRef = useRef(tab.sql);

  const run = useMutation({
    mutationFn: ({ confirmed }: { confirmed: boolean }) =>
      ipc.executeSql(tab.connId, sqlRef.current, confirmed),
    onSuccess: (r) => {
      setResult(r);
      setPendingDanger(null);
    },
    onError: (e) => {
      if (isAppError(e) && e.kind === "dangerousStatement") {
        setPendingDanger(e.message);
      }
    },
  });

  const execute = () => {
    if (sqlRef.current.trim() === "") return;
    run.mutate({ confirmed: false });
  };

  const onMount: OnMount = (editor) => {
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      if (sqlRef.current.trim() !== "") {
        run.mutate({ confirmed: false });
      }
    });
    editor.focus();
  };

  const error = run.error && !pendingDanger ? errorMessage(run.error) : null;

  return (
    <div className="flex h-full flex-col">
      <div className="h-2/5 min-h-32 border-b">
        <Editor
          defaultLanguage="sql"
          defaultValue={tab.sql}
          onMount={onMount}
          onChange={(v) => {
            sqlRef.current = v ?? "";
            updateSql(tab.id, sqlRef.current);
          }}
          options={{
            minimap: { enabled: false },
            fontSize: 13,
            lineNumbers: "on",
            scrollBeyondLastLine: false,
            automaticLayout: true,
          }}
        />
      </div>

      <div className="flex h-8 items-center gap-3 border-b px-2 text-xs">
        <Button size="sm" className="h-6 px-2 text-xs" onClick={execute} disabled={run.isPending}>
          Run (Cmd+Enter)
        </Button>
        {run.isPending && <span className="text-muted-foreground">Running...</span>}
        {result && (
          <span className="text-muted-foreground">
            {result.affectedRows !== null
              ? `${result.affectedRows} rows affected`
              : `${result.rows.length} rows${result.truncated ? " (truncated at 10000)" : ""}`}
            {" in "}
            {result.durationMs} ms
          </span>
        )}
      </div>

      <div className="min-h-0 flex-1">
        {error && <div className="p-3 text-sm text-red-500 whitespace-pre-wrap">{error}</div>}
        {!error && result && result.columns.length > 0 && (
          <ResultGrid columns={result.columns} rows={result.rows} />
        )}
        {!error && result && result.columns.length === 0 && (
          <div className="p-3 text-sm text-muted-foreground">Statement executed.</div>
        )}
        {!error && !result && (
          <div className="p-3 text-sm text-muted-foreground">Press Cmd+Enter to run the query.</div>
        )}
      </div>

      <Dialog open={pendingDanger !== null} onOpenChange={(open) => !open && setPendingDanger(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Dangerous statement</DialogTitle>
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
    </div>
  );
}
