import type { Tab } from "@/stores/workspace";

export function SqlEditorTab({ tab }: { tab: Extract<Tab, { kind: "sql" }> }) {
  return <div className="p-4 text-sm text-muted-foreground">SQL tab: {tab.title}</div>;
}
