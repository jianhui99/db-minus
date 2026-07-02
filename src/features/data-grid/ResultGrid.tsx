import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  flexRender, getCoreRowModel, useReactTable, type ColumnDef,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ArrowDown, ArrowUp } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { CellValue, ColumnMeta, Sort } from "@/lib/ipc";

function formatCell(v: CellValue): string {
  if (v === null) return "";
  if (typeof v === "object") return JSON.stringify(v);
  return String(v);
}

type Selection = { row: number; col: number | null } | null;

interface Props {
  columns: ColumnMeta[];
  rows: CellValue[][];
  onEndReached?: () => void;
  sort?: Sort | null;
  onSortChange?: (sort: Sort | null) => void;
}

const ROW_HEIGHT = 28;

export function ResultGrid({ columns, rows, onEndReached, sort, onSortChange }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [selection, setSelection] = useState<Selection>(null);

  const columnDefs = useMemo<ColumnDef<CellValue[]>[]>(
    () =>
      columns.map((c, i) => ({
        id: String(i),
        header: c.name,
        accessorFn: (row) => row[i],
        size: 160,
        minSize: 60,
      })),
    [columns],
  );

  const table = useReactTable({
    data: rows,
    columns: columnDefs,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
  });

  const tableRows = table.getRowModel().rows;

  const virtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const virtualItems = virtualizer.getVirtualItems();

  useEffect(() => {
    const last = virtualItems.at(-1);
    if (last && last.index >= tableRows.length - 50) {
      onEndReached?.();
    }
  }, [virtualItems, tableRows.length, onEndReached]);

  const copySelection = async () => {
    if (!selection) return;
    const row = rows[selection.row];
    if (!row) return;
    const text =
      selection.col === null
        ? row.map(formatCell).join("\t")
        : formatCell(row[selection.col]);
    await writeText(text);
  };

  const cycleSort = (columnName: string) => {
    if (!onSortChange) return;
    if (sort?.column !== columnName) onSortChange({ column: columnName, desc: false });
    else if (!sort.desc) onSortChange({ column: columnName, desc: true });
    else onSortChange(null);
  };

  const headerGroups = table.getHeaderGroups();
  const totalWidth = table.getTotalSize();

  return (
    <div
      ref={containerRef}
      tabIndex={0}
      className="h-full overflow-auto outline-none"
      onKeyDown={(e) => {
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "c") {
          e.preventDefault();
          void copySelection();
        }
      }}
    >
      <div style={{ width: totalWidth + 48 }}>
        <div className="sticky top-0 z-10 flex border-b bg-background">
          <div className="w-12 shrink-0 border-r bg-muted/40" />
          {headerGroups[0]?.headers.map((header) => {
            const name = columns[Number(header.id)]?.name ?? "";
            const active = sort?.column === name;
            return (
              <div
                key={header.id}
                style={{ width: header.getSize() }}
                className="relative flex shrink-0 cursor-pointer select-none items-center gap-1 border-r px-2 py-1 text-xs font-medium hover:bg-accent"
                onClick={() => cycleSort(name)}
              >
                <span className="truncate">{flexRender(header.column.columnDef.header, header.getContext())}</span>
                {active && (sort!.desc ? <ArrowDown className="size-3" /> : <ArrowUp className="size-3" />)}
                <div
                  onMouseDown={header.getResizeHandler()}
                  onTouchStart={header.getResizeHandler()}
                  onClick={(e) => e.stopPropagation()}
                  className="absolute right-0 top-0 h-full w-1 cursor-col-resize hover:bg-primary/50"
                />
              </div>
            );
          })}
        </div>

        <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
          {virtualItems.map((vi) => {
            const row = tableRows[vi.index];
            return (
              <div
                key={vi.key}
                className="absolute left-0 flex w-full border-b"
                style={{ top: vi.start, height: ROW_HEIGHT }}
              >
                <button
                  className={
                    "w-12 shrink-0 border-r bg-muted/40 text-right text-xs text-muted-foreground px-1 " +
                    (selection?.row === vi.index && selection.col === null ? "bg-primary/20" : "")
                  }
                  onClick={() => setSelection({ row: vi.index, col: null })}
                >
                  {vi.index + 1}
                </button>
                {row.getVisibleCells().map((cell, ci) => {
                  const value = rows[vi.index]?.[ci] ?? null;
                  const selected = selection?.row === vi.index && selection.col === ci;
                  return (
                    <div
                      key={cell.id}
                      style={{ width: cell.column.getSize() }}
                      className={
                        "shrink-0 truncate border-r px-2 py-1 text-xs " +
                        (selected ? "bg-primary/20 " : "") +
                        (value === null ? "italic text-muted-foreground" : "")
                      }
                      onClick={() => setSelection({ row: vi.index, col: ci })}
                    >
                      {value === null ? "NULL" : formatCell(value)}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
