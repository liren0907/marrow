import { workspace } from "./workspace.svelte";
import { tabPeekRegistry } from "./tabRegistry.svelte";
import { toggleBottomPanel } from "$lib/panels/bottomPanelState.svelte";
import { toggleSearch } from "$lib/search/searchState.svelte";
import { toggleCommandPalette } from "$lib/command/commandPaletteState.svelte";
import { toggleTweaks } from "$lib/settings/tweaksState.svelte";
import { toggleSettings } from "$lib/settings/settingsModalState.svelte";

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
    // Cmd+P — toggle full-text search modal
    if (k === "p" && !e.shiftKey) {
      e.preventDefault();
      toggleSearch();
      return;
    }
    // Cmd+Shift+P — toggle command palette
    if (k === "p" && e.shiftKey) {
      e.preventDefault();
      toggleCommandPalette();
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
    // Cmd+Shift+Space — peek at cursor in active markdown tab
    if ((k === " " || e.code === "Space") && e.shiftKey) {
      e.preventDefault();
      const pane = workspace.activePane;
      if (pane.activeTabId) tabPeekRegistry.get(pane.activeTabId)?.();
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
    // Cmd+Shift+, — open Settings page (full overlay, all tabs)
    if (k === "," && e.shiftKey) {
      e.preventDefault();
      toggleSettings();
      return;
    }
    // Cmd+, — open tweaks (theme + accent quick popover)
    if (k === ",") {
      e.preventDefault();
      toggleTweaks();
      return;
    }
  };
  window.addEventListener("keydown", handler, true);
  return () => window.removeEventListener("keydown", handler, true);
}
