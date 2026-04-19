import { workspace } from "$lib/workspace/workspace.svelte";
import { tabPeekRegistry } from "$lib/workspace/tabRegistry.svelte";
import {
  bottomPanel,
  toggleBottomPanel,
} from "$lib/panels/bottomPanelState.svelte";
import { toggleSearch } from "$lib/search/searchState.svelte";
import { peek } from "$lib/peek/peekState.svelte";
import {
  toggleBreadcrumb,
  togglePaneOutline,
} from "$lib/settings/uiSettings.svelte";
import { openRecentWorkspacePicker } from "$lib/workspace/recentWorkspacePickerState.svelte";
import { openTweaks } from "$lib/settings/tweaksState.svelte";

export interface Command {
  id: string;
  title: string;
  category: string;
  shortcut?: string;
  action: () => void;
}

export function getCommands(): Command[] {
  return [
    // Navigation
    {
      id: "search-workspace",
      title: "Search in workspace",
      category: "Navigation",
      shortcut: "⌘P / ⇧⌘F",
      action: toggleSearch,
    },
    {
      id: "open-graph",
      title: "Open graph view",
      category: "Navigation",
      shortcut: "⇧⌘G",
      action: () => workspace.openGraph(),
    },
    {
      id: "peek-at-cursor",
      title: "Peek at cursor",
      category: "Navigation",
      shortcut: "⇧⌘Space",
      action: () => {
        const pane = workspace.activePane;
        if (pane.activeTabId) tabPeekRegistry.get(pane.activeTabId)?.();
      },
    },
    {
      id: "clear-peek",
      title: "Clear peek stack",
      category: "Navigation",
      action: () => peek.clear(),
    },

    // View
    {
      id: "toggle-bottom-panel",
      title: "Toggle bottom panel",
      category: "View",
      shortcut: "⌘J",
      action: toggleBottomPanel,
    },
    {
      id: "show-backlinks",
      title: "Show backlinks",
      category: "View",
      action: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "backlinks";
      },
    },
    {
      id: "show-unresolved",
      title: "Show unresolved links",
      category: "View",
      action: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "unresolved";
      },
    },
    {
      id: "show-tags",
      title: "Show tags",
      category: "View",
      action: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "tags";
      },
    },
    {
      id: "show-outline",
      title: "Show document outline",
      category: "View",
      action: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "outline";
      },
    },
    {
      id: "show-peek",
      title: "Show peek panel",
      category: "View",
      action: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "peek";
      },
    },
    {
      id: "toggle-breadcrumb",
      title: "Toggle editor breadcrumb",
      category: "View",
      action: toggleBreadcrumb,
    },
    {
      id: "toggle-pane-outline",
      title: "Toggle pane outline (right gutter)",
      category: "View",
      action: togglePaneOutline,
    },
    {
      id: "open-tweaks",
      title: "Open tweaks (theme + accent)",
      category: "View",
      shortcut: "⌘,",
      action: openTweaks,
    },

    // Workspace
    {
      id: "workspace.openRecent",
      title: "Open recent workspace…",
      category: "Workspace",
      action: () => queueMicrotask(openRecentWorkspacePicker),
    },
    {
      id: "split-pane",
      title: "Split pane",
      category: "Workspace",
      shortcut: "⌘\\",
      action: () => workspace.splitPane(),
    },
    {
      id: "focus-pane-1",
      title: "Focus pane 1",
      category: "Workspace",
      shortcut: "⌘1",
      action: () => workspace.focusPaneByIndex(0),
    },
    {
      id: "focus-pane-2",
      title: "Focus pane 2",
      category: "Workspace",
      shortcut: "⌘2",
      action: () => workspace.focusPaneByIndex(1),
    },

    // Editor
    {
      id: "close-tab",
      title: "Close current tab",
      category: "Editor",
      shortcut: "⌘W",
      action: () => {
        const pane = workspace.activePane;
        if (pane.activeTabId) workspace.closeTab(pane.id, pane.activeTabId);
      },
    },
    {
      id: "next-tab",
      title: "Next tab",
      category: "Editor",
      shortcut: "⇧⌘]",
      action: () => workspace.nextTab(),
    },
    {
      id: "prev-tab",
      title: "Previous tab",
      category: "Editor",
      shortcut: "⇧⌘[",
      action: () => workspace.prevTab(),
    },
  ];
}
