<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { openDirectoryDialog } from "$lib/workspace/tauri";
  import FileTree from "$lib/tree/FileTree.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";

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
    workspace.close();
  }
</script>

<div
  class="flex flex-col bg-base-100 border-r border-base-200 h-full"
  style="width: {width}px"
>
  <!-- Header -->
  <div
    class="flex items-center gap-2 px-3 py-2.5 border-b border-base-200 shrink-0"
  >
    <span
      class="material-symbols-rounded text-primary text-[20px] shrink-0"
      >folder_open</span
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
    <button
      onclick={toggleSidebar}
      class="btn btn-ghost btn-xs btn-square text-base-content/40 hover:text-base-content shrink-0"
      title="Collapse sidebar"
    >
      <span class="material-symbols-rounded text-[18px]">first_page</span>
    </button>
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
        onclick={closeWorkspace}
        class="btn btn-ghost btn-xs flex-1 justify-start text-base-content/60 font-normal"
        title="Close workspace"
      >
        <span class="material-symbols-rounded text-[16px]">close</span>
        <span class="text-xs">Close workspace</span>
      </button>
      <button
        onclick={() => workspace.openGraph()}
        class="btn btn-ghost btn-xs btn-square text-base-content/60"
        title="Open graph view (⌘⇧G)"
        aria-label="Open graph view"
      >
        <span class="material-symbols-rounded text-[16px]">hub</span>
      </button>
    {:else}
      <div class="flex-1"></div>
    {/if}
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
