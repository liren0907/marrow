// Modal state for the Settings page (full-screen overlay reachable from
// the activity bar's gear icon and ⇧⌘,). Independent from TweaksPanel —
// they share the same underlying stores but expose different surfaces.

export type SettingsTab = "appearance" | "editor" | "workspace" | "about";

const TABS: SettingsTab[] = ["appearance", "editor", "workspace", "about"];

interface SettingsModalState {
  isOpen: boolean;
  activeTab: SettingsTab;
}

export const settingsModal = $state<SettingsModalState>({
  isOpen: false,
  activeTab: "appearance",
});

export function openSettings(tab?: SettingsTab): void {
  settingsModal.isOpen = true;
  if (tab) settingsModal.activeTab = tab;
}

export function closeSettings(): void {
  settingsModal.isOpen = false;
}

export function toggleSettings(): void {
  settingsModal.isOpen = !settingsModal.isOpen;
}

export function setSettingsTab(tab: SettingsTab): void {
  if (!TABS.includes(tab)) return;
  settingsModal.activeTab = tab;
}
