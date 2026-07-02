import { create } from "zustand";

interface UiState {
  connectionsOpen: boolean;
  quickOpenOpen: boolean;
  importOpen: boolean;
  setConnectionsOpen: (open: boolean) => void;
  setQuickOpenOpen: (open: boolean) => void;
  setImportOpen: (open: boolean) => void;
}

export const useUi = create<UiState>((set) => ({
  connectionsOpen: false,
  quickOpenOpen: false,
  importOpen: false,
  setConnectionsOpen: (open) => set({ connectionsOpen: open }),
  setQuickOpenOpen: (open) => set({ quickOpenOpen: open }),
  setImportOpen: (open) => set({ importOpen: open }),
}));
