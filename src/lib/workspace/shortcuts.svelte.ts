import { workspace } from "./workspace.svelte";
import { quickOpen } from "$lib/quickopen/quickOpenState.svelte";
import { toggleBottomPanel } from "$lib/panels/bottomPanelState.svelte";
import { toggleSearch } from "$lib/search/searchState.svelte";

// Per-tab save dispatcher. Tab viewers (e.g. MarkdownTab) register their
// save fn on mount and unregister on destroy. The global Cmd+S handler
// looks up the active tab's entry and invokes it.
export const tabSaveRegistry = new Map<string, () => void | Promise<void>>();

export function registerTabSave(
  tabId: string,
  fn: () => void | Promise<void>,
): void {
  tabSaveRegistry.set(tabId, fn);
}

export function unregisterTabSave(tabId: string): void {
  tabSaveRegistry.delete(tabId);
}

function saveActive(): void {
  const pane = workspace.activePane;
  if (!pane.activeTabId) return;
  const fn = tabSaveRegistry.get(pane.activeTabId);
  if (fn) void fn();
}

export function initShortcuts(): () => void {
  const handler = (e: KeyboardEvent) => {
    const meta = e.metaKey || e.ctrlKey;
    if (!meta) return;
    const k = e.key.toLowerCase();

    // Cmd+S — save active tab
    if (k === "s" && !e.shiftKey) {
      e.preventDefault();
      saveActive();
      return;
    }
    // Cmd+W — close active tab
    if (k === "w" && !e.shiftKey) {
      e.preventDefault();
      const pane = workspace.activePane;
      if (pane.activeTabId) workspace.closeTab(pane.id, pane.activeTabId);
      return;
    }
    // Cmd+P — toggle quick-open
    if (k === "p" && !e.shiftKey) {
      e.preventDefault();
      quickOpen.isOpen = !quickOpen.isOpen;
      if (quickOpen.isOpen) {
        quickOpen.query = "";
        quickOpen.selectedIdx = 0;
      }
      return;
    }
    // Cmd+\ — split pane
    if (k === "\\") {
      e.preventDefault();
      workspace.splitPane();
      return;
    }
    // Cmd+J — toggle bottom panel (backlinks / unresolved / tags)
    if (k === "j" && !e.shiftKey) {
      e.preventDefault();
      toggleBottomPanel();
      return;
    }
    // Cmd+Shift+F — toggle full-text search modal
    if (k === "f" && e.shiftKey) {
      e.preventDefault();
      toggleSearch();
      return;
    }
    // Cmd+Shift+G — open backlink graph view
    if (k === "g" && e.shiftKey) {
      e.preventDefault();
      workspace.openGraph();
      return;
    }
    // Cmd+1 / Cmd+2 — focus pane by index
    if (k === "1") {
      e.preventDefault();
      workspace.focusPaneByIndex(0);
      return;
    }
    if (k === "2") {
      e.preventDefault();
      workspace.focusPaneByIndex(1);
      return;
    }
    // Cmd+Shift+] / Cmd+Shift+[ — next/prev tab in active pane
    if (e.shiftKey && (k === "]" || e.code === "BracketRight")) {
      e.preventDefault();
      workspace.nextTab();
      return;
    }
    if (e.shiftKey && (k === "[" || e.code === "BracketLeft")) {
      e.preventDefault();
      workspace.prevTab();
      return;
    }
  };
  window.addEventListener("keydown", handler, true);
  return () => window.removeEventListener("keydown", handler, true);
}
