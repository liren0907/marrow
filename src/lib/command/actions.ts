// Single source of truth for every user-triggerable action in Marrow.
//
// Each action is defined ONCE here — id, title, category, default key
// binding(s), and a `run` thunk. Three consumers derive from this registry:
//   - the keyboard matcher       (workspace/shortcuts.svelte.ts)
//   - the Command Palette        (command/commands.ts → CommandPalette.svelte)
//   - the Keyboard settings page (settings/sections/KeyboardSection.svelte)
//
// Key bindings are canonical combo strings — see `eventToCombo`.

import { workspace } from "$lib/workspace/workspace.svelte";
import { isConvertible } from "$lib/workspace/fileKind";
import {
  tabPeekRegistry,
  tabSaveRegistry,
} from "$lib/workspace/tabRegistry.svelte";
import {
  bottomPanel,
  toggleBottomPanel,
} from "$lib/panels/bottomPanelState.svelte";
import { toggleSearch } from "$lib/search/searchState.svelte";
import { peek } from "$lib/peek/peekState.svelte";
import {
  toggleBreadcrumb,
  togglePaneOutline,
  toggleSidebar,
} from "$lib/settings/uiSettings.svelte";
import { openRecentWorkspacePicker } from "$lib/workspace/recentWorkspacePickerState.svelte";
import { toggleTweaks } from "$lib/settings/tweaksState.svelte";
import { toggleCommandPalette } from "$lib/command/commandPaletteState.svelte";
import { toggleSettings } from "$lib/settings/settingsModalState.svelte";

// ─── Key-combo representation ────────────────────────────────────────────
// A binding is a canonical string: "Mod" (= ⌘ on mac / Ctrl elsewhere),
// then optional "Shift" / "Alt", then exactly one physical
// `KeyboardEvent.code`, joined by "+" — e.g. "Mod+KeyS",
// "Mod+Shift+BracketRight". Every binding MUST include "Mod" so shortcuts
// never fire while the user is typing.

const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
]);

export function isModifierCode(code: string): boolean {
  return MODIFIER_CODES.has(code);
}

/** Normalize a keydown into a canonical combo, or null if it is not a
 * candidate shortcut (no ⌘/Ctrl held, or only a modifier pressed). */
export function eventToCombo(e: KeyboardEvent): string | null {
  if (!(e.metaKey || e.ctrlKey)) return null;
  if (isModifierCode(e.code)) return null;
  const parts = ["Mod"];
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");
  parts.push(e.code);
  return parts.join("+");
}

const CODE_LABELS: Record<string, string> = {
  Slash: "/",
  Backslash: "\\",
  Comma: ",",
  Period: ".",
  Semicolon: ";",
  Quote: "'",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
};

function codeLabel(code: string): string {
  const mapped = CODE_LABELS[code];
  if (mapped) return mapped;
  if (code.startsWith("Key")) return code.slice(3); // KeyP → P
  if (code.startsWith("Digit")) return code.slice(5); // Digit1 → 1
  if (code.startsWith("Numpad")) return `Num${code.slice(6)}`;
  return code;
}

function isMac(): boolean {
  return typeof navigator !== "undefined" && /Mac/i.test(navigator.userAgent);
}

/** Render a canonical combo for display — "⇧⌘P" on mac, "Ctrl+Shift+P"
 * elsewhere. */
export function comboToDisplay(combo: string): string {
  const mac = isMac();
  const out: string[] = [];
  for (const part of combo.split("+")) {
    if (part === "Mod") out.push(mac ? "⌘" : "Ctrl");
    else if (part === "Shift") out.push(mac ? "⇧" : "Shift");
    else if (part === "Alt") out.push(mac ? "⌥" : "Alt");
    else out.push(codeLabel(part));
  }
  return mac ? out.join("") : out.join("+");
}

/** Combos owned by the text editors (Milkdown / CodeMirror) or the OS.
 * The shortcut recorder refuses these — the capture-phase global handler
 * would otherwise steal them. Value = human-readable owner. */
export const RESERVED: Record<string, string> = {
  "Mod+KeyB": "Bold (editor)",
  "Mod+KeyI": "Italic (editor)",
  "Mod+KeyU": "Underline (editor)",
  "Mod+KeyZ": "Undo (editor)",
  "Mod+Shift+KeyZ": "Redo (editor)",
  "Mod+KeyY": "Redo (editor)",
  "Mod+KeyA": "Select all",
  "Mod+KeyC": "Copy",
  "Mod+KeyV": "Paste",
  "Mod+KeyX": "Cut",
  "Mod+KeyQ": "Quit",
};

// ─── Action registry ─────────────────────────────────────────────────────

export type ActionCategory = "Navigation" | "View" | "Workspace" | "Editor";

export interface ActionDef {
  id: string;
  title: string;
  category: ActionCategory;
  /** Default key combo(s). Empty = no default shortcut. */
  defaultKeys: string[];
  run: () => void;
  /** Shown in the Command Palette unless explicitly false. */
  inPalette?: boolean;
}

/** The full action list — the single source of truth. */
export function getActions(): ActionDef[] {
  return [
    // ─── Navigation ───
    {
      id: "search-workspace",
      title: "Search in workspace",
      category: "Navigation",
      defaultKeys: ["Mod+KeyP", "Mod+Shift+KeyF"],
      run: toggleSearch,
    },
    {
      id: "command-palette-toggle",
      title: "Toggle command palette",
      category: "Navigation",
      defaultKeys: ["Mod+Shift+KeyP"],
      inPalette: false,
      run: toggleCommandPalette,
    },
    {
      id: "open-graph",
      title: "Open graph view",
      category: "Navigation",
      defaultKeys: ["Mod+Shift+KeyG"],
      run: () => workspace.openGraph(),
    },
    {
      id: "peek-at-cursor",
      title: "Peek at cursor",
      category: "Navigation",
      defaultKeys: ["Mod+Shift+Space"],
      run: () => {
        const pane = workspace.activePane;
        if (pane.activeTabId) tabPeekRegistry.get(pane.activeTabId)?.();
      },
    },
    {
      id: "clear-peek",
      title: "Clear peek stack",
      category: "Navigation",
      defaultKeys: [],
      run: () => peek.clear(),
    },

    // ─── View ───
    {
      id: "toggle-sidebar",
      title: "Toggle sidebar",
      category: "View",
      defaultKeys: ["Mod+Shift+KeyB"],
      run: toggleSidebar,
    },
    {
      id: "toggle-bottom-panel",
      title: "Toggle bottom panel",
      category: "View",
      defaultKeys: ["Mod+KeyJ"],
      run: toggleBottomPanel,
    },
    {
      id: "show-backlinks",
      title: "Show backlinks",
      category: "View",
      defaultKeys: [],
      run: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "backlinks";
      },
    },
    {
      id: "show-unresolved",
      title: "Show unresolved links",
      category: "View",
      defaultKeys: [],
      run: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "unresolved";
      },
    },
    {
      id: "show-tags",
      title: "Show tags",
      category: "View",
      defaultKeys: [],
      run: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "tags";
      },
    },
    {
      id: "show-outline",
      title: "Show document outline",
      category: "View",
      defaultKeys: [],
      run: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "outline";
      },
    },
    {
      id: "show-peek",
      title: "Show peek panel",
      category: "View",
      defaultKeys: [],
      run: () => {
        bottomPanel.isOpen = true;
        bottomPanel.activeTab = "peek";
      },
    },
    {
      id: "toggle-breadcrumb",
      title: "Toggle editor breadcrumb",
      category: "View",
      defaultKeys: [],
      run: toggleBreadcrumb,
    },
    {
      id: "toggle-pane-outline",
      title: "Toggle pane outline (right gutter)",
      category: "View",
      defaultKeys: [],
      run: togglePaneOutline,
    },
    {
      id: "open-tweaks",
      title: "Open tweaks (theme + accent)",
      category: "View",
      defaultKeys: ["Mod+Comma"],
      run: toggleTweaks,
    },

    // ─── Workspace ───
    {
      id: "workspace.openRecent",
      title: "Open recent workspace…",
      category: "Workspace",
      defaultKeys: [],
      run: () => queueMicrotask(openRecentWorkspacePicker),
    },
    {
      id: "open-settings",
      title: "Open settings",
      category: "Workspace",
      defaultKeys: ["Mod+Shift+Comma"],
      run: toggleSettings,
    },
    {
      id: "split-pane",
      title: "Split pane",
      category: "Workspace",
      defaultKeys: ["Mod+Backslash"],
      run: () => workspace.splitPane(),
    },
    {
      id: "focus-pane-1",
      title: "Focus pane 1",
      category: "Workspace",
      defaultKeys: ["Mod+Digit1"],
      run: () => workspace.focusPaneByIndex(0),
    },
    {
      id: "focus-pane-2",
      title: "Focus pane 2",
      category: "Workspace",
      defaultKeys: ["Mod+Digit2"],
      run: () => workspace.focusPaneByIndex(1),
    },

    // ─── Editor ───
    {
      id: "save-active-tab",
      title: "Save current tab",
      category: "Editor",
      defaultKeys: ["Mod+KeyS"],
      run: () => {
        const pane = workspace.activePane;
        if (!pane.activeTabId) return;
        const fn = tabSaveRegistry.get(pane.activeTabId);
        if (fn) void fn();
      },
    },
    {
      id: "close-tab",
      title: "Close current tab",
      category: "Editor",
      defaultKeys: ["Mod+KeyW"],
      run: () => {
        const pane = workspace.activePane;
        if (pane.activeTabId) workspace.closeTab(pane.id, pane.activeTabId);
      },
    },
    {
      id: "next-tab",
      title: "Next tab",
      category: "Editor",
      defaultKeys: ["Mod+Shift+BracketRight"],
      run: () => workspace.nextTab(),
    },
    {
      id: "prev-tab",
      title: "Previous tab",
      category: "Editor",
      defaultKeys: ["Mod+Shift+BracketLeft"],
      run: () => workspace.prevTab(),
    },
    {
      id: "toggle-markdown-view",
      title: "Toggle markdown pretty/raw view",
      category: "Editor",
      defaultKeys: ["Mod+Slash"],
      run: () => {
        const pane = workspace.activePane;
        const tab = pane.tabs.find((t) => t.id === pane.activeTabId);
        if (tab && tab.kind === "markdown") {
          const next =
            (tab.viewMode ?? "pretty") === "pretty" ? "raw" : "pretty";
          workspace.patchTab(tab.id, { viewMode: next });
        }
      },
    },
    {
      id: "convert-to-markdown",
      title: "Convert current file to Markdown…",
      category: "Editor",
      defaultKeys: [],
      run: () => {
        const pane = workspace.activePane;
        const tab = pane.tabs.find((t) => t.id === pane.activeTabId);
        if (tab && isConvertible(tab.path)) {
          workspace.openConvert(tab.path);
        }
      },
    },
  ];
}
