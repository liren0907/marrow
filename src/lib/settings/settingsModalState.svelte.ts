// Settings now lives as a regular workspace tab (kind: "settings", virtual
// path "marrow://settings"). This module is the public API for opening /
// closing / toggling that tab plus the section-nav state used inside
// SettingsTab.svelte.
//
// File name kept as `settingsModalState` for now to minimize import churn
// across the codebase — the export names (`openSettings`, etc.) are still
// the right verbs, only the underlying mechanism changed (modal → tab).
//
// SECTION NAV STATE: `settingsModal.activeTab` survives across tab close /
// reopen so users get back to the section they were on. Persisting it to
// localStorage isn't worth the bytes — re-opening Settings within the same
// session is the common case.

import { workspace } from "$lib/workspace/workspace.svelte";

export type SettingsTab =
  | "appearance"
  | "editor"
  | "workspace"
  | "advanced"
  | "about";

const TABS: SettingsTab[] = [
  "appearance",
  "editor",
  "workspace",
  "advanced",
  "about",
];

interface SectionNavState {
  activeTab: SettingsTab;
}

// Exported as `settingsModal` (legacy name) so existing consumer imports in
// SettingsTab.svelte still read naturally without a rename.
export const settingsModal = $state<SectionNavState>({
  activeTab: "appearance",
});

/**
 * Open the Settings tab — focuses the existing one if already open in any
 * pane, otherwise creates a new tab in the active pane. Optionally jumps
 * straight to a specific section.
 */
export function openSettings(tab?: SettingsTab): void {
  workspace.openSettingsTab();
  if (tab) settingsModal.activeTab = tab;
}

/**
 * Close the Settings tab if open. No-op otherwise. Users can also close
 * via the tab's × button or ⌘W like any other tab — this exists for
 * keyboard shortcuts and the close path of toggleSettings().
 */
export function closeSettings(): void {
  workspace.closeSettingsTab();
}

/**
 * Toggle behavior on ⇧⌘,:
 *   - If a Settings tab is the ACTIVE tab in the active pane → close it
 *     (treat ⇧⌘, as "dismiss what I just opened").
 *   - If a Settings tab exists elsewhere or isn't currently focused →
 *     focus / open it.
 */
export function toggleSettings(): void {
  const ap = workspace.activePane;
  const activeIsSettings = ap.tabs.find(
    (t) => t.id === ap.activeTabId && t.kind === "settings",
  );
  if (activeIsSettings) {
    workspace.closeSettingsTab();
    return;
  }
  workspace.openSettingsTab();
}

export function setSettingsTab(tab: SettingsTab): void {
  if (!TABS.includes(tab)) return;
  settingsModal.activeTab = tab;
}
