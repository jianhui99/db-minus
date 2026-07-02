import type { Tab } from "@/stores/workspace";

export function TableDataTab({ tab }: { tab: Extract<Tab, { kind: "table" }> }) {
  return <div className="p-4 text-sm text-muted-foreground">Table: {tab.table}</div>;
}
