import { create } from "zustand";

interface UiState {
  connectionsOpen: boolean;
  quickOpenOpen: boolean;
  setConnectionsOpen: (open: boolean) => void;
  setQuickOpenOpen: (open: boolean) => void;
}

export const useUi = create<UiState>((set) => ({
  connectionsOpen: false,
  quickOpenOpen: false,
  setConnectionsOpen: (open) => set({ connectionsOpen: open }),
  setQuickOpenOpen: (open) => set({ quickOpenOpen: open }),
}));
