import type {
  FileMeta,
  FsEventKind,
  Pane,
  Tab,
  WorkspaceInfo,
} from "./types";
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
  fileIndex: FileMeta[];
  splitRatio: number;
}>({
  info: null,
  panes: [newPane()],
  activePaneId: "",
  fileIndex: [],
  splitRatio: 0.5,
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
  get fileIndex(): FileMeta[] {
    return state.fileIndex;
  },
  get splitRatio(): number {
    return state.splitRatio;
  },

  async open(path: string): Promise<void> {
    const info = await cmd.openWorkspaceCmd(path);
    state.info = info;
    state.panes = [newPane()];
    state.activePaneId = state.panes[0].id;
    state.fileIndex = [];
    void this.refreshFileIndex();
  },

  close(): void {
    state.info = null;
    state.panes = [newPane()];
    state.activePaneId = state.panes[0].id;
    state.fileIndex = [];
  },

  async refreshFileIndex(): Promise<void> {
    if (!state.info) return;
    try {
      state.fileIndex = await cmd.listWorkspaceFiles(state.info.root);
    } catch (e) {
      console.warn("[workspace] file index refresh failed", e);
    }
  },

  resolveBasename(name: string): string | null {
    const target = name.toLowerCase();
    const targetMd = target.endsWith(".md") ? target : `${target}.md`;
    for (const f of state.fileIndex) {
      if (f.kind !== "markdown") continue;
      if (f.name.toLowerCase() === targetMd) return f.path;
    }
    // Fallback: stem match without extension
    for (const f of state.fileIndex) {
      if (f.kind !== "markdown") continue;
      const stem = f.name.replace(/\.[^.]+$/, "").toLowerCase();
      if (stem === target) return f.path;
    }
    return null;
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
    // Collapse empty pane only if there's another pane to fall back to.
    if (pane.tabs.length === 0 && state.panes.length > 1) {
      this.closePane(paneId);
    }
  },

  splitPane(): void {
    if (state.panes.length >= 2) {
      // Already split — focus the other pane.
      const other = state.panes.find((p) => p.id !== state.activePaneId);
      if (other) state.activePaneId = other.id;
      return;
    }
    const sourcePane = findPane(state.activePaneId) ?? state.panes[0];
    const newP = newPane();
    // Copy the active tab into the new pane (don't move it).
    const activeTab = sourcePane.tabs.find((t) => t.id === sourcePane.activeTabId);
    if (activeTab) {
      const copy: Tab = {
        id: crypto.randomUUID(),
        path: activeTab.path,
        kind: activeTab.kind,
        title: activeTab.title,
        isDirty: false,
      };
      newP.tabs.push(copy);
      newP.activeTabId = copy.id;
    }
    state.panes.push(newP);
    state.activePaneId = newP.id;
  },

  closePane(paneId: string): void {
    if (state.panes.length <= 1) return;
    const idx = state.panes.findIndex((p) => p.id === paneId);
    if (idx < 0) return;
    state.panes.splice(idx, 1);
    if (state.activePaneId === paneId) {
      state.activePaneId = state.panes[0].id;
    }
  },

  setSplitRatio(r: number): void {
    state.splitRatio = Math.max(0.15, Math.min(0.85, r));
  },

  focusPaneByIndex(index: number): void {
    const pane = state.panes[index];
    if (pane) state.activePaneId = pane.id;
  },

  nextTab(): void {
    const pane = findPane(state.activePaneId);
    if (!pane || pane.tabs.length === 0) return;
    const idx = pane.tabs.findIndex((t) => t.id === pane.activeTabId);
    const next = pane.tabs[(idx + 1) % pane.tabs.length];
    pane.activeTabId = next.id;
  },

  prevTab(): void {
    const pane = findPane(state.activePaneId);
    if (!pane || pane.tabs.length === 0) return;
    const idx = pane.tabs.findIndex((t) => t.id === pane.activeTabId);
    const prev = pane.tabs[(idx - 1 + pane.tabs.length) % pane.tabs.length];
    pane.activeTabId = prev.id;
  },

  replaceCurrentTab(path: string): void {
    const pane = findPane(state.activePaneId);
    if (!pane || !pane.activeTabId) {
      this.openFile(path);
      return;
    }
    const tab = pane.tabs.find((t) => t.id === pane.activeTabId);
    if (!tab) {
      this.openFile(path);
      return;
    }
    // If the same file is already open in another tab, just focus it.
    const existing = pane.tabs.find((t) => t.path === path);
    if (existing) {
      pane.activeTabId = existing.id;
      return;
    }
    tab.id = crypto.randomUUID();
    tab.path = path;
    tab.kind = classifyFile(path);
    tab.title = basename(path);
    tab.isDirty = false;
    tab.lastKnownMtime = undefined;
    tab.reloadToken = undefined;
    tab.missing = false;
    pane.activeTabId = tab.id;
  },

  openInOtherPane(path: string): void {
    if (state.panes.length < 2) {
      // Create the second pane (don't copy current tab — we want it to host
      // the new file fresh).
      state.panes.push(newPane());
    }
    const otherPane = state.panes.find((p) => p.id !== state.activePaneId);
    if (!otherPane) {
      this.openFile(path);
      return;
    }
    const existing = otherPane.tabs.find((t) => t.path === path);
    if (existing) {
      otherPane.activeTabId = existing.id;
    } else {
      const tab: Tab = {
        id: crypto.randomUUID(),
        path,
        kind: classifyFile(path),
        title: basename(path),
        isDirty: false,
      };
      otherPane.tabs.push(tab);
      otherPane.activeTabId = tab.id;
    }
    state.activePaneId = otherPane.id;
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

  notifyExternalChange(path: string, kind: FsEventKind): void {
    for (const pane of state.panes) {
      for (const tab of pane.tabs) {
        if (tab.path !== path) continue;
        if (kind === "remove") {
          tab.missing = true;
        } else {
          tab.missing = false;
          tab.reloadToken = (tab.reloadToken ?? 0) + 1;
        }
      }
    }
  },
};
