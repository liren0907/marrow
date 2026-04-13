<script lang="ts">
  import { onMount } from "svelte";
  import type { Tab } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { readTextFile, writeTextFile } from "$lib/workspace/tauri";
  import MilkdownEditor from "$lib/editor/milkdown/MilkdownEditor.svelte";
  import { debounce } from "$lib/utils/debounce";
  import { showError, showWarning } from "$lib/stores/toastStore.svelte";

  let { tab }: { tab: Tab } = $props();

  let loaded = $state(false);
  let initialContent = $state("");
  let loadError = $state<string | null>(null);
  let reloadKey = $state(0);
  let lastHandledToken = 0;

  let currentContent = "";
  let savedContent = "";

  const isActive = $derived(
    workspace.panes.some(
      (p) => p.activeTabId === tab.id && p.tabs.some((t) => t.id === tab.id),
    ),
  );

  async function save() {
    if (currentContent === savedContent) return;
    try {
      const result = await writeTextFile(
        tab.path,
        currentContent,
        tab.lastKnownMtime,
      );
      savedContent = currentContent;
      workspace.patchTab(tab.id, {
        isDirty: false,
        lastKnownMtime: result.mtime,
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(`Failed to save ${tab.title}: ${msg}`);
    }
  }

  const debouncedSave = debounce(() => void save(), 800);

  function handleChange(md: string) {
    currentContent = md;
    const dirty = currentContent !== savedContent;
    if (dirty !== tab.isDirty) {
      workspace.patchTab(tab.id, { isDirty: dirty });
    }
    if (dirty) debouncedSave();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isActive) return;
    if ((e.metaKey || e.ctrlKey) && e.key === "s") {
      e.preventDefault();
      debouncedSave.cancel();
      void save();
    }
  }

  onMount(() => {
    (async () => {
      try {
        const result = await readTextFile(tab.path);
        initialContent = result.content;
        currentContent = result.content;
        savedContent = result.content;
        workspace.patchTab(tab.id, { lastKnownMtime: result.mtime });
        loaded = true;
      } catch (e) {
        loadError = e instanceof Error ? e.message : String(e);
      }
    })();
  });

  $effect(() => {
    const token = tab.reloadToken ?? 0;
    if (!loaded || token === lastHandledToken) return;
    lastHandledToken = token;
    if (tab.isDirty) {
      showWarning(`${tab.title} changed on disk — still editing, not reloaded`);
      return;
    }
    (async () => {
      try {
        const result = await readTextFile(tab.path);
        initialContent = result.content;
        currentContent = result.content;
        savedContent = result.content;
        workspace.patchTab(tab.id, {
          isDirty: false,
          lastKnownMtime: result.mtime,
        });
        reloadKey++;
      } catch (e) {
        showError(
          `Failed to reload ${tab.title}: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    })();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="w-full h-full relative">
  {#if loadError}
    <div class="p-6 text-error text-sm">Failed to load: {loadError}</div>
  {:else if loaded}
    {#key reloadKey}
      <MilkdownEditor initial={initialContent} onChange={handleChange} />
    {/key}
  {:else}
    <div class="p-6 text-base-content/40 text-sm">Loading…</div>
  {/if}
</div>
