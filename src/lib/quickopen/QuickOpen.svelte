<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";
  import type { FileMeta } from "$lib/workspace/types";
  import { quickOpen } from "./quickOpenState.svelte";
  import { fuzzyMatch } from "./fuzzy";

  interface Result {
    file: FileMeta;
    score: number;
    nameIndices: number[];
  }

  const KIND_ICON: Record<string, string> = {
    markdown: "description",
    image: "image",
    video: "videocam",
    audio: "graphic_eq",
    pdf: "picture_as_pdf",
    text: "code",
    unsupported: "draft",
  };

  let inputEl: HTMLInputElement | undefined = $state();

  const results = $derived.by(() => {
    const q = quickOpen.query;
    if (!q) {
      // No query: show first 30 files alphabetically by name.
      return workspace.fileIndex
        .slice()
        .sort((a, b) => a.name.localeCompare(b.name))
        .slice(0, 30)
        .map((file) => ({ file, score: 0, nameIndices: [] }) satisfies Result);
    }
    const out: Result[] = [];
    for (const file of workspace.fileIndex) {
      const m = fuzzyMatch(q, file.name);
      if (!m) continue;
      out.push({ file, score: m.score, nameIndices: m.indices });
    }
    out.sort((a, b) => b.score - a.score || a.file.name.localeCompare(b.file.name));
    return out.slice(0, 50);
  });

  $effect(() => {
    if (quickOpen.isOpen) {
      // Reset selection when results change.
      if (quickOpen.selectedIdx >= results.length) quickOpen.selectedIdx = 0;
      // Focus input shortly after open.
      queueMicrotask(() => inputEl?.focus());
    }
  });

  function close() {
    quickOpen.isOpen = false;
  }

  function pick(idx: number, newTab: boolean) {
    const r = results[idx];
    if (!r) return;
    if (newTab) {
      workspace.openFile(r.file.path);
    } else {
      workspace.replaceCurrentTab(r.file.path);
    }
    close();
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (results.length === 0) {
      if (e.key === "Escape") close();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      quickOpen.selectedIdx = (quickOpen.selectedIdx + 1) % results.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      quickOpen.selectedIdx =
        (quickOpen.selectedIdx - 1 + results.length) % results.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(quickOpen.selectedIdx, e.metaKey || e.ctrlKey);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
  }

  function dirOf(path: string): string {
    const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
    if (idx <= 0) return "";
    const root = workspace.info?.root ?? "";
    const dir = path.slice(0, idx);
    if (root && dir.startsWith(root)) {
      return dir.slice(root.length).replace(/^[/\\]/, "") || "·";
    }
    return dir;
  }

  function highlight(name: string, indices: number[]): { ch: string; on: boolean }[] {
    const set = new Set(indices);
    const out: { ch: string; on: boolean }[] = [];
    for (let i = 0; i < name.length; i++) {
      out.push({ ch: name[i], on: set.has(i) });
    }
    return out;
  }
</script>

{#if quickOpen.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Quick open">
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={close}
      aria-label="Close"
    ></button>
    <div
      class="modal-box max-w-2xl bg-base-100 border border-base-300 shadow-2xl flex flex-col p-0 max-h-[70vh] overflow-hidden"
    >
      <div class="px-3 py-2 border-b border-base-200">
        <input
          bind:this={inputEl}
          type="text"
          class="w-full bg-transparent outline-none text-base px-2 py-1"
          placeholder="Search files…"
          bind:value={quickOpen.query}
          onkeydown={onInputKeydown}
        />
      </div>
      <ul class="flex-1 overflow-y-auto py-1">
        {#if results.length === 0}
          <li class="px-4 py-3 text-sm text-base-content/40 italic">No matches</li>
        {:else}
          {#each results as r, i (r.file.path)}
            <li>
              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-1.5 text-left text-sm hover:bg-base-200"
                class:bg-base-200={i === quickOpen.selectedIdx}
                onmousemove={() => (quickOpen.selectedIdx = i)}
                onclick={(e) => pick(i, e.metaKey || e.ctrlKey)}
              >
                <span
                  class="material-symbols-rounded text-[18px] text-base-content/50 shrink-0"
                  >{KIND_ICON[r.file.kind] ?? "draft"}</span
                >
                <span class="truncate flex-1">
                  {#each highlight(r.file.name, r.nameIndices) as part}
                    {#if part.on}
                      <span class="text-primary font-semibold">{part.ch}</span>
                    {:else}
                      {part.ch}
                    {/if}
                  {/each}
                </span>
                <span class="text-[11px] text-base-content/40 truncate max-w-[40%]"
                  >{dirOf(r.file.path)}</span
                >
              </button>
            </li>
          {/each}
        {/if}
      </ul>
      <div
        class="px-3 py-1.5 border-t border-base-200 text-[10px] text-base-content/40 flex gap-3"
      >
        <span>↑↓ select</span>
        <span>Enter open</span>
        <span>⌘Enter new tab</span>
        <span>Esc close</span>
      </div>
    </div>
  </div>
{/if}
