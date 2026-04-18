<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import Sidebar from "./Sidebar.svelte";
  import Toast from "$lib/components/ui/Toast.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { initFsEvents } from "$lib/workspace/fsEvents";
  import { initShortcuts } from "$lib/workspace/shortcuts.svelte";
  import QuickOpen from "$lib/quickopen/QuickOpen.svelte";
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

  let isSidebarExpanded = $state(true);
  let sidebarWidth = $state(256);
  let isResizing = $state(false);
  let dragOver = $state(false);
  let isLg = $state(true);
  let isDrawerOpen = $state(false);

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
      const newWidth = Math.min(
        MAX_SIDEBAR_WIDTH,
        Math.max(MIN_SIDEBAR_WIDTH, e.clientX),
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

    const mq = window.matchMedia("(min-width: 1024px)");
    const syncLg = () => {
      isLg = mq.matches;
      if (isLg) isDrawerOpen = false;
    };
    syncLg();
    mq.addEventListener("change", syncLg);

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
      mq.removeEventListener("change", syncLg);
    };
  });

  function toggleSidebar() {
    if (isLg) {
      isSidebarExpanded = !isSidebarExpanded;
    } else {
      isDrawerOpen = !isDrawerOpen;
    }
  }
</script>

<div
  class="app drawer h-screen"
  class:lg:drawer-open={isSidebarExpanded}
  class:resizing={isResizing}
  style:--sidebar-width="{sidebarWidth}px"
>
  <input
    id="sidebar-drawer"
    type="checkbox"
    class="drawer-toggle"
    bind:checked={isDrawerOpen}
  />

  <div class="drawer-content flex flex-col min-h-0 min-w-0">
    <main class="flex-1 flex flex-col min-h-0 min-w-0 bg-base-100 relative">
      {#if (isLg && !isSidebarExpanded) || (!isLg && !isDrawerOpen)}
        <div class="absolute top-2 left-2 z-30">
          <button
            onclick={toggleSidebar}
            class="btn btn-circle btn-ghost btn-xs shadow-sm bg-base-100 hover:bg-base-200 border border-base-200"
            title="Open Sidebar"
          >
            <span class="material-symbols-rounded text-base-content/70 text-[18px]">
              menu
            </span>
          </button>
        </div>
      {/if}

      <div class="flex-1 flex flex-col min-h-0 min-w-0">
        {@render children()}
      </div>

      {#if bottomPanel.isOpen}
        <BottomPanel />
      {/if}

      {#if dragOver}
        <div
          class="absolute inset-0 pointer-events-none bg-primary/10 border-4 border-dashed border-primary flex items-center justify-center z-50"
        >
          <div
            class="bg-base-100 px-4 py-2 rounded-[var(--mw-radius-sm)] shadow-lg text-primary font-medium text-sm"
          >
            Drop folder to open as workspace
          </div>
        </div>
      {/if}
    </main>
  </div>

  <div class="drawer-side z-40">
    <label
      for="sidebar-drawer"
      aria-label="close sidebar"
      class="drawer-overlay"
    ></label>
    <Sidebar {isSidebarExpanded} {toggleSidebar} width={sidebarWidth} />
  </div>

  {#if isSidebarExpanded}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle hidden lg:block"
      onmousedown={handleResizeStart}
      style="left: {sidebarWidth}px"
      class:active={isResizing}
    ></div>
  {/if}
</div>

<Toast />
<QuickOpen />
<SearchModal />
<CommandPalette />
<ConflictModal />
<ContextMenu />
<NamePromptModal />
<RenameModal />
<FileHistoryModal />
<RecentWorkspacePicker />

<style>
  .app.resizing {
    user-select: none;
    cursor: col-resize;
  }

  @media (min-width: 1024px) {
    :global(.app.lg\:drawer-open > .drawer-side) {
      width: var(--sidebar-width, 256px);
    }
  }

  .resize-handle {
    position: fixed;
    top: 0;
    bottom: 0;
    width: 6px;
    transform: translateX(-3px);
    cursor: col-resize;
    z-index: 50;
    transition: background-color 0.15s;
  }

  .resize-handle:hover,
  .resize-handle.active {
    background-color: color-mix(in oklch, var(--color-primary) 30%, transparent);
  }
</style>
