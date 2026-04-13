import type { Pane, Tab, WorkspaceInfo } from "./types";
import { classifyFile, basename } from "./fileKind";
import * as cmd from "./tauri";

function newPane(): Pane {
  return {
    id: crypto.randomUUID(),
    tabs: [],
    activeTabId: null,
  };
}

const state = $state<{
  info: WorkspaceInfo | null;
  panes: Pane[];
  activePaneId: string;
}>({
  info: null,
  panes: [newPane()],
  activePaneId: "",
});
state.activePaneId = state.panes[0].id;

function findPane(paneId: string): Pane | undefined {
  return state.panes.find((p) => p.id === paneId);
}

export const workspace = {
  get info() {
    return state.info;
  },
  get panes() {
    return state.panes;
  },
  get activePaneId() {
    return state.activePaneId;
  },
  get activePane(): Pane {
    return findPane(state.activePaneId) ?? state.panes[0];
  },

  async open(path: string): Promise<void> {
    const info = await cmd.openWorkspaceCmd(path);
    state.info = info;
    state.panes = [newPane()];
    state.activePaneId = state.panes[0].id;
  },

  close(): void {
    state.info = null;
    state.panes = [newPane()];
    state.activePaneId = state.panes[0].id;
  },

  openFile(path: string, paneId?: string): void {
    const pane = findPane(paneId ?? state.activePaneId);
    if (!pane) return;

    const existing = pane.tabs.find((t) => t.path === path);
    if (existing) {
      pane.activeTabId = existing.id;
      return;
    }

    const tab: Tab = {
      id: crypto.randomUUID(),
      path,
      kind: classifyFile(path),
      title: basename(path),
      isDirty: false,
    };
    pane.tabs.push(tab);
    pane.activeTabId = tab.id;
  },

  closeTab(paneId: string, tabId: string): void {
    const pane = findPane(paneId);
    if (!pane) return;
    const idx = pane.tabs.findIndex((t) => t.id === tabId);
    if (idx < 0) return;
    pane.tabs.splice(idx, 1);
    if (pane.activeTabId === tabId) {
      const next = pane.tabs[Math.min(idx, pane.tabs.length - 1)];
      pane.activeTabId = next?.id ?? null;
    }
  },

  setActiveTab(paneId: string, tabId: string): void {
    const pane = findPane(paneId);
    if (pane) pane.activeTabId = tabId;
  },

  setActivePane(paneId: string): void {
    if (findPane(paneId)) state.activePaneId = paneId;
  },

  patchTab(tabId: string, updates: Partial<Tab>): void {
    for (const pane of state.panes) {
      const tab = pane.tabs.find((t) => t.id === tabId);
      if (tab) {
        Object.assign(tab, updates);
        return;
      }
    }
  },
};
