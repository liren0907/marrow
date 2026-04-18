<script lang="ts">
  import {
    recentWorkspacePicker,
    closeRecentWorkspacePicker,
  } from "$lib/workspace/recentWorkspacePickerState.svelte";
  import {
    listRecentWorkspaces,
    forgetWorkspace,
    pathExists,
    type WorkspaceSummary,
  } from "$lib/workspace/tauri";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";
  import { formatRelative } from "$lib/utils/formatRelative";

  let inputEl: HTMLInputElement | undefined = $state();
  let items = $state<WorkspaceSummary[]>([]);

  async function refresh() {
    try {
      items = await listRecentWorkspaces(50);
    } catch (e) {
      console.warn("[RecentWorkspacePicker] fetch failed", e);
      items = [];
    }
  }

  $effect(() => {
    if (recentWorkspacePicker.isOpen) {
      void refresh();
      queueMicrotask(() => inputEl?.focus());
    }
  });

  const results = $derived.by(() => {
    const q = recentWorkspacePicker.query.trim().toLowerCase();
    if (!q) return items;
    return items.filter(
      (x) =>
        x.name.toLowerCase().includes(q) ||
        x.last_path.toLowerCase().includes(q),
    );
  });

  $effect(() => {
    if (recentWorkspacePicker.selectedIdx >= results.length) {
      recentWorkspacePicker.selectedIdx = 0;
    }
  });

  async function pick(idx: number) {
    const entry = results[idx];
    if (!entry) return;
    try {
      if (!(await pathExists(entry.last_path))) {
        showError(`"${entry.name}" is no longer available`);
        await forgetWorkspace(entry.id);
        items = items.filter((x) => x.id !== entry.id);
        return;
      }
      await workspace.open(entry.last_path);
      closeRecentWorkspacePicker();
    } catch (e) {
      showError(`Failed to open: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  async function forget(entry: WorkspaceSummary, event: MouseEvent) {
    event.stopPropagation();
    try {
      await forgetWorkspace(entry.id);
      items = items.filter((x) => x.id !== entry.id);
    } catch (e) {
      showError(`Failed to forget: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeRecentWorkspacePicker();
      return;
    }
    if (results.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      recentWorkspacePicker.selectedIdx =
        (recentWorkspacePicker.selectedIdx + 1) % results.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      recentWorkspacePicker.selectedIdx =
        (recentWorkspacePicker.selectedIdx - 1 + results.length) % results.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      void pick(recentWorkspacePicker.selectedIdx);
    }
  }
</script>

{#if recentWorkspacePicker.isOpen}
  <div
    class="modal modal-open z-[60]"
    role="dialog"
    aria-label="Open recent workspace"
  >
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={closeRecentWorkspacePicker}
      aria-label="Close"
    ></button>
    <div
      class="modal-box max-w-2xl bg-base-100 border border-base-300 shadow-2xl flex flex-col p-0 max-h-[70vh] overflow-hidden"
    >
      <div class="px-3 py-2 border-b border-base-200">
        <input
          bind:this={inputEl}
          type="text"
          class="w-full bg-transparent outline-none text-sm px-2 py-1"
          placeholder="Search recent workspaces…"
          bind:value={recentWorkspacePicker.query}
          onkeydown={onInputKeydown}
        />
      </div>
      <ul class="flex-1 overflow-y-auto py-1">
        {#if results.length === 0}
          <li class="px-4 py-3 text-sm text-base-content/40 italic">
            {items.length === 0 ? "No recent workspaces yet" : "No matches"}
          </li>
        {:else}
          {#each results as entry, i (entry.id)}
            <li>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div
                role="button"
                tabindex="-1"
                class="group w-full flex items-center gap-2 px-3 py-1.5 text-left cursor-pointer"
                class:bg-base-200={i === recentWorkspacePicker.selectedIdx}
                onmousemove={() => (recentWorkspacePicker.selectedIdx = i)}
                onclick={() => pick(i)}
              >
                <span
                  class="material-symbols-rounded text-[16px] text-base-content/50 shrink-0"
                  >folder</span
                >
                <div class="flex-1 min-w-0">
                  <div class="text-sm truncate text-base-content">
                    {entry.name}
                  </div>
                  <div
                    class="text-[11px] text-base-content/40 truncate font-mono leading-tight"
                  >
                    {entry.last_path}
                  </div>
                </div>
                <span class="text-[11px] text-base-content/40 shrink-0">
                  {formatRelative(entry.last_opened_ts)}
                </span>
                <button
                  type="button"
                  class="opacity-0 group-hover:opacity-100 text-base-content/40 hover:text-base-content shrink-0 p-0.5 rounded hover:bg-base-300 transition-opacity"
                  onclick={(e) => forget(entry, e)}
                  title="Forget this workspace"
                  aria-label="Forget"
                >
                  <span class="material-symbols-rounded text-[14px]">close</span>
                </button>
              </div>
            </li>
          {/each}
        {/if}
      </ul>
      <div
        class="px-3 py-1.5 border-t border-base-200 text-[10px] text-base-content/40 flex gap-3"
      >
        <span>↑↓ select</span>
        <span>Enter open</span>
        <span>Esc close</span>
      </div>
    </div>
  </div>
{/if}
