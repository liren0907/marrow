<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { Tab } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { readTextFile, writeTextFile } from "$lib/workspace/tauri";
  import {
    registerTabSave,
    unregisterTabSave,
  } from "$lib/workspace/shortcuts.svelte";
  import { openConflict } from "$lib/conflict/conflictState.svelte";
  import MilkdownEditor from "$lib/editor/milkdown/MilkdownEditor.svelte";
  import type { WikiLinkSuggestion } from "$lib/editor/milkdown/wikiLink/suggest";
  import { debounce } from "$lib/utils/debounce";
  import { showError, showWarning } from "$lib/stores/toastStore.svelte";

  function getWikiLinkSuggestions(query: string): WikiLinkSuggestion[] {
    const q = query.toLowerCase();
    const items = workspace.fileIndex
      .filter((f) => f.kind === "markdown")
      .map((f) => {
        const stem = f.name.replace(/\.md$/i, "").toLowerCase();
        let score = -1;
        if (q === "") score = 1;
        else if (stem.startsWith(q)) score = 3;
        else if (stem.includes(q)) score = 2;
        else if (f.path.toLowerCase().includes(q)) score = 1;
        const idx = Math.max(f.path.lastIndexOf("/"), f.path.lastIndexOf("\\"));
        const dir = idx > 0 ? f.path.slice(0, idx) : "";
        const lastSep = Math.max(dir.lastIndexOf("/"), dir.lastIndexOf("\\"));
        const folder = lastSep > 0 ? dir.slice(lastSep + 1) : dir;
        return { score, name: f.name, path: f.path, folder };
      })
      .filter((f) => f.score >= 0)
      .sort((a, b) => b.score - a.score || a.name.localeCompare(b.name));
    return items.slice(0, 12);
  }

  function isWikiLinkResolved(target: string): boolean {
    return workspace.resolveBasename(target) !== null;
  }

  function handleWikiLinkClick(
    target: string,
    mods: { meta: boolean; shift: boolean; middle: boolean },
  ): void {
    const path = workspace.resolveBasename(target);
    if (!path) return;
    if (mods.meta && mods.shift) {
      workspace.openInOtherPane(path);
    } else if (mods.meta || mods.middle) {
      workspace.openFile(path);
    } else {
      workspace.replaceCurrentTab(path);
    }
  }

  let { tab }: { tab: Tab } = $props();

  let loaded = $state(false);
  let initialContent = $state("");
  let loadError = $state<string | null>(null);
  let reloadKey = $state(0);
  let lastHandledToken = 0;

  let currentContent = "";
  let savedContent = "";

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
      if (msg.includes("File changed on disk")) {
        void openConflict(tab, currentContent);
      } else {
        showError(`Failed to save ${tab.title}: ${msg}`);
      }
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

  function saveNow(): Promise<void> {
    debouncedSave.cancel();
    return save();
  }

  onMount(() => {
    registerTabSave(tab.id, saveNow);
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

  onDestroy(() => {
    unregisterTabSave(tab.id);
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

<div class="w-full h-full relative">
  {#if loadError}
    <div class="p-6 text-error text-sm">Failed to load: {loadError}</div>
  {:else if loaded}
    {#key reloadKey}
      <MilkdownEditor
        initial={initialContent}
        onChange={handleChange}
        onWikiLinkClick={handleWikiLinkClick}
        {getWikiLinkSuggestions}
        {isWikiLinkResolved}
      />
    {/key}
  {:else}
    <div class="p-6 text-base-content/40 text-sm">Loading…</div>
  {/if}
</div>
