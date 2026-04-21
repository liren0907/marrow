<script lang="ts">
  import { onMount } from "svelte";
  import {
    listRecentWorkspaces,
    forgetWorkspace,
    pathExists,
    type WorkspaceSummary,
  } from "$lib/workspace/tauri";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { showError } from "$lib/stores/toastStore.svelte";
  import { formatRelative } from "$lib/utils/formatRelative";
  import SectionLabel from "$lib/components/ui/SectionLabel.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  let items = $state<WorkspaceSummary[]>([]);
  let loading = $state(true);

  async function refresh() {
    try {
      items = await listRecentWorkspaces(10);
    } catch (e) {
      console.warn("[RecentList] fetch failed", e);
      items = [];
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function openEntry(entry: WorkspaceSummary) {
    try {
      if (!(await pathExists(entry.last_path))) {
        showError(`"${entry.name}" is no longer available`);
        await forgetWorkspace(entry.id);
        items = items.filter((x) => x.id !== entry.id);
        return;
      }
      await workspace.open(entry.last_path);
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
</script>

{#if !loading && items.length > 0}
  <div class="w-full max-w-md mt-6 flex flex-col gap-1">
    <div class="px-1 pb-1 flex items-center justify-between">
      <SectionLabel>Recent</SectionLabel>
    </div>
    <div class="flex flex-col">
      {#each items as entry (entry.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          role="button"
          tabindex="0"
          class="group flex items-center gap-2 w-full px-2 py-1.5 rounded-[var(--mw-radius-xs)] hover:bg-base-200/60 text-left transition-colors cursor-pointer"
          onclick={() => openEntry(entry)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              openEntry(entry);
            }
          }}
          title={entry.last_path}
        >
          <Icon name="folder" size={16} class="text-base-content/50" />
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium truncate text-base-content">
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
            <Icon name="x" size={14} />
          </button>
        </div>
      {/each}
    </div>
  </div>
{/if}
