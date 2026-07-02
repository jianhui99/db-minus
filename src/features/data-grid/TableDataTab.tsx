import { useInfiniteQuery } from "@tanstack/react-query";
import { useCallback, useMemo, useState } from "react";
import { errorMessage, ipc, type Cursor, type Sort } from "@/lib/ipc";
import { useWorkspace, type Tab } from "@/stores/workspace";
import { ResultGrid } from "./ResultGrid";

const PAGE_SIZE = 500;

export function TableDataTab({ tab }: { tab: Extract<Tab, { kind: "table" }> }) {
  const [sort, setSort] = useState<Sort | null>(null);
  const refreshNonce = useWorkspace((s) => s.refreshNonce);

  const query = useInfiniteQuery({
    queryKey: ["tablePage", tab.connId, tab.namespace, tab.table, sort, refreshNonce],
    queryFn: ({ pageParam }) =>
      ipc.fetchTablePage(tab.connId, {
        namespace: tab.namespace,
        table: tab.table,
        sort,
        cursor: pageParam,
        limit: PAGE_SIZE,
      }),
    initialPageParam: null as Cursor | null,
    getNextPageParam: (last) => last.nextCursor,
  });

  const columns = query.data?.pages[0]?.columns ?? [];
  const rows = useMemo(
    () => query.data?.pages.flatMap((p) => p.rows) ?? [],
    [query.data],
  );

  const onEndReached = useCallback(() => {
    if (query.hasNextPage && !query.isFetchingNextPage) {
      void query.fetchNextPage();
    }
  }, [query]);

  if (query.isLoading) {
    return <div className="p-4 text-sm text-muted-foreground">Loading {tab.table}...</div>;
  }
  if (query.error) {
    return <div className="p-4 text-sm text-red-500">{errorMessage(query.error)}</div>;
  }

  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1">
        <ResultGrid
          columns={columns}
          rows={rows}
          sort={sort}
          onSortChange={setSort}
          onEndReached={onEndReached}
        />
      </div>
      <div className="flex h-7 items-center gap-3 border-t px-2 text-xs text-muted-foreground">
        <span>
          {rows.length} rows loaded{query.hasNextPage ? " (scroll for more)" : ""}
        </span>
        {query.isFetchingNextPage && <span>Loading more...</span>}
      </div>
    </div>
  );
}
