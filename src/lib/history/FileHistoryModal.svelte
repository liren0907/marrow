<script lang="ts">
  import { diffLines } from "diff";
  import MarkdownIt from "markdown-it";
  import {
    fileHistory,
    closeFileHistory,
    selectSnapshot,
    setViewMode,
    restoreSelected,
  } from "./fileHistoryState.svelte";

  const md = new MarkdownIt({ html: false, linkify: true, breaks: true });
  const MAX_DIFF_LINES = 5000;

  const selected = $derived(fileHistory.entries[fileHistory.selectedIdx] ?? null);
  const isRenameEntry = $derived(selected?.op === "rename");

  function lineCount(s: string): number {
    let n = 0;
    for (let i = 0; i < s.length; i++) if (s[i] === "\n") n++;
    return n + 1;
  }

  const diffParts = $derived.by(() => {
    if (fileHistory.viewMode !== "diff") return null;
    if (isRenameEntry) return null;
    const selectedText = fileHistory.selectedContent;
    const currentText = fileHistory.currentContent;
    if (selectedText == null) return null;
    if (lineCount(selectedText) + lineCount(currentText) > MAX_DIFF_LINES * 2) {
      return { tooLarge: true, parts: [] as ReturnType<typeof diffLines> };
    }
    return { tooLarge: false, parts: diffLines(selectedText, currentText) };
  });

  const renderedMarkdown = $derived.by(() => {
    if (fileHistory.viewMode !== "preview") return "";
    if (isRenameEntry) return "";
    if (fileHistory.selectedContent == null) return "";
    return md.render(fileHistory.selectedContent);
  });

  function formatTimestamp(unix: number): string {
    const d = new Date(unix * 1000);
    return d.toLocaleString();
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  let confirmRestore = $state(false);

  function requestRestore() {
    confirmRestore = true;
  }
  async function doRestore() {
    confirmRestore = false;
    await restoreSelected();
  }
  function cancelRestore() {
    confirmRestore = false;
  }

  function handleKey(e: KeyboardEvent) {
    if (!fileHistory.isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      closeFileHistory();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = Math.min(
        fileHistory.selectedIdx + 1,
        fileHistory.entries.length - 1,
      );
      void selectSnapshot(next);
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      const next = Math.max(fileHistory.selectedIdx - 1, 0);
      void selectSnapshot(next);
      return;
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

{#if fileHistory.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="File history">
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={closeFileHistory}
      aria-label="Close"
    ></button>
    <div
      class="modal-box bg-base-100 border border-base-300 shadow-2xl flex flex-col p-0 overflow-hidden max-w-5xl w-11/12 h-[80vh]"
    >
      <div class="px-5 py-3 border-b border-base-200 flex items-center justify-between">
        <div>
          <h3 class="font-semibold text-base">
            History — {fileHistory.targetTitle ?? ""}
          </h3>
          <p class="text-xs text-base-content/60 mt-0.5">
            {fileHistory.entries.length} revision{fileHistory.entries.length === 1 ? "" : "s"}
          </p>
        </div>
        <button
          type="button"
          class="btn btn-sm btn-ghost btn-square"
          onclick={closeFileHistory}
          aria-label="Close"
        >
          <span class="material-symbols-rounded icon-sm">close</span>
        </button>
      </div>

      <div class="flex-1 flex min-h-0">
        <!-- Left: revision list -->
        <div class="w-64 border-r border-base-200 overflow-y-auto shrink-0">
          {#if fileHistory.isLoading}
            <div class="p-4 text-sm text-base-content/50">Loading…</div>
          {:else if fileHistory.entries.length === 0}
            <div class="p-4 text-sm text-base-content/50">
              No history yet. Save the file to record a revision.
            </div>
          {:else}
            <ul class="flex flex-col py-1">
              {#each fileHistory.entries as entry, i (entry.ts + entry.hash)}
                <li>
                  <button
                    type="button"
                    class="w-full text-left px-3 py-2 text-xs border-l-2 hover:bg-base-200 {i === fileHistory.selectedIdx
                      ? 'bg-base-200 border-primary'
                      : 'border-transparent'}"
                    onclick={() => selectSnapshot(i)}
                  >
                    <div class="flex items-center gap-1.5">
                      {#if entry.op === "save"}
                        <span class="badge badge-sm badge-primary badge-soft">save</span>
                      {:else if entry.op === "restore"}
                        <span class="badge badge-sm badge-info badge-soft">restore</span>
                      {:else if entry.op === "rename"}
                        <span
                          class="badge badge-sm badge-warning badge-soft"
                          title={entry.prev_path ?? ""}
                        >
                          rename
                        </span>
                      {/if}
                      <span class="text-base-content/50 ml-auto">
                        {formatSize(entry.size)}
                      </span>
                    </div>
                    <div class="mt-1 text-[11px] text-base-content/70">
                      {formatTimestamp(entry.ts)}
                    </div>
                    {#if entry.op === "rename" && entry.prev_path}
                      <div class="mt-0.5 text-[10px] text-base-content/40 truncate">
                        from {entry.prev_path}
                      </div>
                    {/if}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <!-- Right: preview / diff -->
        <div class="flex-1 flex flex-col min-w-0">
          {#if fileHistory.entries.length > 0 && !isRenameEntry}
            <div
              class="flex items-center gap-1 px-4 py-2 border-b border-base-200 bg-base-200/40"
            >
              <button
                type="button"
                class="btn btn-xs {fileHistory.viewMode === 'preview'
                  ? 'btn-primary'
                  : 'btn-ghost'}"
                onclick={() => setViewMode("preview")}
              >
                Preview
              </button>
              <button
                type="button"
                class="btn btn-xs {fileHistory.viewMode === 'diff'
                  ? 'btn-primary'
                  : 'btn-ghost'}"
                onclick={() => setViewMode("diff")}
              >
                Diff vs current
              </button>
            </div>
          {/if}

          <div class="flex-1 overflow-auto">
            {#if fileHistory.entries.length === 0}
              <div class="p-6 text-sm text-base-content/50">
                Nothing selected.
              </div>
            {:else if isRenameEntry}
              <div class="p-6 text-sm text-base-content/60">
                <span class="badge badge-warning badge-soft mb-2">rename</span>
                <div>
                  Renamed from <span class="font-mono">{selected?.prev_path ?? "?"}</span>
                </div>
                <div class="text-xs text-base-content/40 mt-2">
                  No content captured for rename events.
                </div>
              </div>
            {:else if fileHistory.isSelectedLoading}
              <div class="p-6 text-sm text-base-content/50">Loading snapshot…</div>
            {:else if fileHistory.viewMode === "preview"}
              <div class="prose prose-sm max-w-none p-6 font-sans">
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html renderedMarkdown}
              </div>
            {:else if fileHistory.viewMode === "diff"}
              <div class="px-4 py-3 font-mono text-xs">
                {#if diffParts?.tooLarge}
                  <div class="text-base-content/40 italic">
                    Diff too large to display ({MAX_DIFF_LINES}+ lines)
                  </div>
                {:else if diffParts}
                  {#each diffParts.parts as part}
                    {#if part.added}
                      <div class="text-success bg-success/10 whitespace-pre-wrap">
                        {part.value.replace(/\n$/, "")}
                      </div>
                    {:else if part.removed}
                      <div class="text-error bg-error/10 whitespace-pre-wrap">
                        {part.value.replace(/\n$/, "")}
                      </div>
                    {:else}
                      <div class="text-base-content/50 whitespace-pre-wrap">
                        {part.value.replace(/\n$/, "")}
                      </div>
                    {/if}
                  {/each}
                {/if}
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div
        class="px-5 py-3 bg-base-200/50 border-t border-base-200 flex justify-between items-center gap-3"
      >
        <div class="text-xs text-base-content/50">
          {#if selected && !isRenameEntry}
            <span class="font-mono">{selected.hash.slice(0, 12)}…</span>
          {/if}
        </div>
        <div class="flex items-center gap-2">
          {#if confirmRestore}
            <span class="text-xs text-base-content/70">Overwrite current file?</span>
            <button
              type="button"
              class="btn btn-sm btn-ghost"
              onclick={cancelRestore}
              disabled={fileHistory.isRestoring}
            >
              Cancel
            </button>
            <button
              type="button"
              class="btn btn-sm btn-warning"
              onclick={doRestore}
              disabled={fileHistory.isRestoring}
            >
              {fileHistory.isRestoring ? "Restoring…" : "Confirm restore"}
            </button>
          {:else}
            <button
              type="button"
              class="btn btn-sm btn-ghost"
              onclick={closeFileHistory}
            >
              Close
            </button>
            <button
              type="button"
              class="btn btn-sm btn-primary"
              onclick={requestRestore}
              disabled={!selected || isRenameEntry || fileHistory.isLoading}
            >
              Restore this version
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
