<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    openDirectoryDialog,
    listRecentWorkspaces,
    pathExists,
    forgetWorkspace,
    type WorkspaceSummary,
  } from "$lib/workspace/tauri";
  import FileTree from "$lib/tree/FileTree.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";
  import { openRecentWorkspacePicker } from "$lib/workspace/recentWorkspacePickerState.svelte";

  let {
    isSidebarExpanded = true,
    toggleSidebar = () => {},
    width = 256,
  }: {
    isSidebarExpanded?: boolean;
    toggleSidebar?: () => void;
    width?: number;
  } = $props();

  let isDark = $state(false);

  $effect(() => {
    const t = document.documentElement.getAttribute("data-theme") ?? "";
    isDark = t === "marrow-pro-dark" || t === "dark";
  });

  function toggleTheme() {
    isDark = !isDark;
    const theme = isDark ? "marrow-pro-dark" : "marrow-pro-light";
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
  }

  async function pickFolder() {
    closeSwitcher();
    try {
      const path = await openDirectoryDialog();
      if (path) await workspace.open(path);
    } catch (e) {
      showError(
        `Failed to open folder: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }

  function closeWorkspace() {
    closeSwitcher();
    workspace.close();
  }

  // Switcher dropdown
  let switcherOpen = $state(false);
  let switcherItems = $state<WorkspaceSummary[]>([]);
  let switcherAnchorEl: HTMLDivElement | undefined = $state();
  let switcherMenuEl: HTMLDivElement | undefined = $state();

  async function openSwitcher() {
    try {
      const recents = await listRecentWorkspaces(10);
      const currentRoot = workspace.info?.root ?? "";
      switcherItems = recents.filter((r) => r.last_path !== currentRoot).slice(0, 8);
    } catch {
      switcherItems = [];
    }
    switcherOpen = true;
  }

  function closeSwitcher() {
    switcherOpen = false;
  }

  function toggleSwitcher() {
    if (switcherOpen) closeSwitcher();
    else void openSwitcher();
  }

  async function switchTo(entry: WorkspaceSummary) {
    closeSwitcher();
    try {
      if (!(await pathExists(entry.last_path))) {
        showError(`"${entry.name}" is no longer available`);
        await forgetWorkspace(entry.id);
        return;
      }
      await workspace.open(entry.last_path);
    } catch (e) {
      showError(`Failed to open: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  function openRecentPicker() {
    closeSwitcher();
    queueMicrotask(openRecentWorkspacePicker);
  }

  function onWindowMouseDown(e: MouseEvent) {
    if (!switcherOpen) return;
    const target = e.target as Node;
    if (switcherMenuEl?.contains(target)) return;
    if (switcherAnchorEl?.contains(target)) return;
    closeSwitcher();
  }

  function onWindowKeyDown(e: KeyboardEvent) {
    if (switcherOpen && e.key === "Escape") closeSwitcher();
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={onWindowKeyDown} />

<div
  class="flex flex-col bg-base-100 border-r border-base-200 h-full"
  style="width: {width}px"
>
  <!-- Header -->
  <div
    bind:this={switcherAnchorEl}
    class="flex items-center gap-2 px-3 py-2.5 border-b border-base-200 shrink-0 relative"
  >
    <span
      class="material-symbols-rounded text-primary text-[20px] shrink-0"
      >folder_open</span
    >
    <button
      type="button"
      class="flex-1 min-w-0 text-left flex items-center gap-1 rounded-[var(--mw-radius-xs)] hover:bg-base-200/60 px-1 py-0.5 -mx-1 -my-0.5"
      onclick={toggleSwitcher}
      title="Switch workspace"
    >
      <div class="flex-1 min-w-0">
        {#if workspace.info}
          <div
            class="text-sm font-semibold truncate"
            title={workspace.info.root}
          >
            {workspace.info.name}
          </div>
          <div class="text-[10px] text-base-content/40 truncate font-mono">
            {workspace.info.root}
          </div>
        {:else}
          <span class="text-sm font-semibold text-base-content/60"
            >No workspace</span
          >
        {/if}
      </div>
      <span
        class="material-symbols-rounded text-[16px] text-base-content/40 shrink-0"
      >
        {switcherOpen ? "expand_less" : "expand_more"}
      </span>
    </button>
    <button
      onclick={toggleSidebar}
      class="btn btn-ghost btn-xs btn-square text-base-content/40 hover:text-base-content shrink-0"
      title="Collapse sidebar"
      aria-label="Collapse sidebar"
    >
      <span class="material-symbols-rounded text-[18px]">first_page</span>
    </button>

    {#if switcherOpen}
      <div
        bind:this={switcherMenuEl}
        class="absolute left-3 right-3 top-full mt-1 z-40 rounded-[var(--mw-radius-sm)] border border-base-300 bg-base-100 shadow-lg py-1"
      >
        {#if switcherItems.length > 0}
          <div class="px-2 pt-0.5 pb-1">
            <span
              class="text-[10px] font-bold text-base-content/40 uppercase tracking-wider"
              >Switch to</span
            >
          </div>
          {#each switcherItems as entry (entry.id)}
            <button
              type="button"
              class="w-full flex items-center gap-2 px-2 py-1 text-left hover:bg-base-200/70"
              onclick={() => switchTo(entry)}
              title={entry.last_path}
            >
              <span
                class="material-symbols-rounded text-[14px] text-base-content/50 shrink-0"
                >folder</span
              >
              <span class="text-xs truncate flex-1">{entry.name}</span>
            </button>
          {/each}
          <div class="my-1 border-t border-base-200"></div>
        {/if}
        <button
          type="button"
          class="w-full flex items-center gap-2 px-2 py-1 text-left hover:bg-base-200/70"
          onclick={pickFolder}
        >
          <span
            class="material-symbols-rounded text-[14px] text-base-content/50 shrink-0"
            >folder_open</span
          >
          <span class="text-xs">Open folder…</span>
        </button>
        <button
          type="button"
          class="w-full flex items-center gap-2 px-2 py-1 text-left hover:bg-base-200/70"
          onclick={openRecentPicker}
        >
          <span
            class="material-symbols-rounded text-[14px] text-base-content/50 shrink-0"
            >history</span
          >
          <span class="text-xs">Open recent workspace…</span>
        </button>
        {#if workspace.info}
          <div class="my-1 border-t border-base-200"></div>
          <button
            type="button"
            class="w-full flex items-center gap-2 px-2 py-1 text-left hover:bg-base-200/70"
            onclick={closeWorkspace}
          >
            <span
              class="material-symbols-rounded text-[14px] text-base-content/50 shrink-0"
              >close</span
            >
            <span class="text-xs">Close workspace</span>
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Workspace actions (when no workspace) -->
  {#if !workspace.info}
    <div class="px-3 py-3 border-b border-base-200">
      <button
        onclick={pickFolder}
        class="btn btn-sm btn-primary w-full"
        title="Open a folder as workspace"
      >
        <span class="material-symbols-rounded text-[16px]">folder_open</span>
        Open folder…
      </button>
    </div>
  {/if}

  <!-- File Tree -->
  <div class="flex-1 min-h-0 overflow-y-auto tree-scroll">
    <FileTree />
  </div>

  <!-- Footer -->
  <div
    class="border-t border-base-200 px-2 py-1.5 flex items-center gap-1 shrink-0"
  >
    {#if workspace.info}
      <button
        onclick={() => workspace.openGraph()}
        class="btn btn-ghost btn-xs btn-square text-base-content/60"
        title="Open graph view (⌘⇧G)"
        aria-label="Open graph view"
      >
        <span class="material-symbols-rounded text-[16px]">hub</span>
      </button>
    {/if}
    <div class="flex-1"></div>
    <button
      onclick={toggleTheme}
      class="btn btn-ghost btn-xs btn-square text-base-content/60"
      title={isDark ? "Switch to light" : "Switch to dark"}
    >
      <span class="material-symbols-rounded text-[16px]">
        {isDark ? "dark_mode" : "light_mode"}
      </span>
    </button>
  </div>
</div>

<style>
  .tree-scroll {
    scrollbar-width: none;
  }
  .tree-scroll::-webkit-scrollbar {
    display: none;
  }
</style>
