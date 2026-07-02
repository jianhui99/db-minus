import { X } from "lucide-react";
import { useWorkspace } from "@/stores/workspace";

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, closeTab } = useWorkspace();

  return (
    <div className="flex h-9 items-end gap-px overflow-x-auto border-b bg-muted/40 px-1">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={
            "group flex h-8 max-w-48 cursor-pointer items-center gap-1 rounded-t border border-b-0 px-2 text-sm " +
            (tab.id === activeTabId ? "bg-background" : "bg-muted/60 text-muted-foreground")
          }
          onClick={() => setActiveTab(tab.id)}
        >
          <span className="truncate">{tab.title}</span>
          <button
            className="rounded p-0.5 opacity-0 hover:bg-accent group-hover:opacity-100"
            onClick={(e) => {
              e.stopPropagation();
              closeTab(tab.id);
            }}
          >
            <X className="size-3" />
          </button>
        </div>
      ))}
    </div>
  );
}
