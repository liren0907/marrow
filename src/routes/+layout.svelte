<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import Sidebar from "./Sidebar.svelte";
  import Toast from "$lib/components/ui/Toast.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { initFsEvents } from "$lib/workspace/fsEvents";
  import { initShortcuts } from "$lib/workspace/shortcuts.svelte";
  import SearchModal from "$lib/search/SearchModal.svelte";
  import CommandPalette from "$lib/command/CommandPalette.svelte";
  import ConflictModal from "$lib/conflict/ConflictModal.svelte";
  import BottomPanel from "$lib/panels/BottomPanel.svelte";
  import { bottomPanel } from "$lib/panels/bottomPanelState.svelte";
  import ContextMenu from "$lib/components/ui/ContextMenu.svelte";
  import NamePromptModal from "$lib/tree/NamePromptModal.svelte";
  import RenameModal from "$lib/tree/RenameModal.svelte";
  import FileHistoryModal from "$lib/history/FileHistoryModal.svelte";
  import RecentWorkspacePicker from "$lib/workspace/RecentWorkspacePicker.svelte";
  import TitleBar from "$lib/chrome/TitleBar.svelte";
  import ActivityBar from "$lib/chrome/ActivityBar.svelte";
  import StatusBar from "$lib/chrome/StatusBar.svelte";
  import TweaksPanel from "$lib/settings/TweaksPanel.svelte";
  import { initAccent } from "$lib/settings/accentState.svelte";
  import {
    listRecentWorkspaces,
    forgetWorkspace,
    pathExists,
  } from "$lib/workspace/tauri";
  import {
    showError,
    showSuccess,
    showWarning,
  } from "$lib/stores/toastStore.svelte";
  import "../app.css";
  import "katex/dist/katex.min.css";

  let { children } = $props();

  let sidebarWidth = $state(256);
  let isResizing = $state(false);
  let dragOver = $state(false);

  const MIN_SIDEBAR_WIDTH = 200;
  const MAX_SIDEBAR_WIDTH = 480;

  async function autoReopenRecent() {
    if (workspace.info) return;
    try {
      const recents = await listRecentWorkspaces(1);
      const top = recents[0];
      if (!top) return;
      if (workspace.info) return;
      if (!(await pathExists(top.last_path))) {
        showWarning(`Last workspace "${top.name}" is no longer available`);
        await forgetWorkspace(top.id);
        return;
      }
      await workspace.open(top.last_path);
    } catch (e) {
      console.warn("[layout] auto-reopen failed", e);
    }
  }

  function handleResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;

    function onMouseMove(e: MouseEvent) {
      // Subtract the activity bar width so the sidebar column lines up with
      // the cursor's position within the main-row grid.
      const activityBarW = 44;
      const newWidth = Math.min(
        MAX_SIDEBAR_WIDTH,
        Math.max(MIN_SIDEBAR_WIDTH, e.clientX - activityBarW),
      );
      sidebarWidth = newWidth;
    }

    function onMouseUp() {
      isResizing = false;
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
  }

  onMount(() => {
    const legacyMap: Record<string, string> = {
      light: "marrow-pro-light",
      dark: "marrow-pro-dark",
    };
    const stored = localStorage.getItem("theme");
    const savedTheme = stored ? (legacyMap[stored] ?? stored) : "marrow-pro-light";
    document.documentElement.setAttribute("data-theme", savedTheme);
    if (stored && legacyMap[stored]) localStorage.setItem("theme", savedTheme);

    initAccent();

    let unlisten: (() => void) | null = null;
    let unlistenFs: (() => void) | null = null;
    const unlistenShortcuts = initShortcuts();
    void autoReopenRecent();
    (async () => {
      try {
        unlistenFs = await initFsEvents();
      } catch (e) {
        console.warn("[layout] fs-event listener failed", e);
      }
    })();
    (async () => {
      try {
        unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
          const p = event.payload;
          if (p.type === "enter" || p.type === "over") {
            dragOver = true;
          } else if (p.type === "leave") {
            dragOver = false;
          } else if (p.type === "drop") {
            dragOver = false;
            if (p.paths.length === 0) return;
            const path = p.paths[0];
            try {
              await workspace.open(path);
              showSuccess(`Workspace: ${path}`);
            } catch (e) {
              const msg = e instanceof Error ? e.message : String(e);
              showError(`Could not open as workspace: ${msg}`);
            }
          }
        });
      } catch (e) {
        console.warn("[layout] drag-drop listener failed", e);
      }
    })();

    return () => {
      if (unlisten) unlisten();
      if (unlistenFs) unlistenFs();
      unlistenShortcuts();
    };
  });
</script>

<div
  class="marrow-root"
  class:resizing={isResizing}
  style:--mw-sidebar-width="{sidebarWidth}px"
>
  <TitleBar />

  <div class="main-row">
    <ActivityBar />
    <Sidebar width={sidebarWidth} />
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle"
      onmousedown={handleResizeStart}
      class:active={isResizing}
    ></div>
    <main class="main-pane">
      <div class="pane-children">
        {@render children()}
      </div>

      {#if bottomPanel.isOpen}
        <BottomPanel />
      {/if}

      {#if dragOver}
        <div class="drag-overlay">
          <div class="drag-overlay-label">
            Drop folder to open as workspace
          </div>
        </div>
      {/if}
    </main>
  </div>

  <StatusBar />
</div>

<Toast />
<SearchModal />
<CommandPalette />
<ConflictModal />
<ContextMenu />
<NamePromptModal />
<RenameModal />
<FileHistoryModal />
<RecentWorkspacePicker />
<TweaksPanel />

<style>
  .marrow-root {
    height: 100vh;
    display: grid;
    grid-template-rows: var(--mw-titlebar-h) 1fr var(--mw-statusbar-h);
    background: var(--color-base-100);
    color: var(--color-base-content);
    overflow: hidden;
  }
  .marrow-root.resizing {
    user-select: none;
    cursor: col-resize;
  }
  .main-row {
    display: grid;
    grid-template-columns: var(--mw-activitybar-w) var(--mw-sidebar-width, 256px) 0 1fr;
    min-height: 0;
    min-width: 0;
  }
  .main-pane {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-base-100);
  }
  .pane-children {
    flex: 1;
    min-height: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .resize-handle {
    width: 6px;
    transform: translateX(-3px);
    cursor: col-resize;
    align-self: stretch;
    transition: background-color 0.15s;
    position: relative;
    z-index: 5;
  }
  .resize-handle:hover,
  .resize-handle.active {
    background-color: color-mix(in oklch, var(--mw-accent) 30%, transparent);
  }
  .drag-overlay {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: color-mix(in oklch, var(--mw-accent) 10%, transparent);
    border: 4px dashed var(--mw-accent);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
  }
  .drag-overlay-label {
    background: var(--color-base-100);
    padding: 8px 16px;
    border-radius: var(--mw-radius-sm);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.2);
    color: var(--mw-accent);
    font-weight: 500;
    font-size: 13px;
  }
</style>
