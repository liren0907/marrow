<script lang="ts">
  import { diffLines } from "diff";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { writeTextFile } from "$lib/workspace/tauri";
  import { showError } from "$lib/stores/toastStore.svelte";
  import { conflict, closeConflict } from "./conflictState.svelte";

  const MAX_DIFF_LINES = 5000;

  function lineCount(s: string): number {
    let n = 0;
    for (let i = 0; i < s.length; i++) if (s[i] === "\n") n++;
    return n + 1;
  }

  const diffParts = $derived.by(() => {
    if (!conflict.showDiff) return null;
    const disk = conflict.diskContent ?? "";
    const mine = conflict.myContent;
    if (lineCount(disk) + lineCount(mine) > MAX_DIFF_LINES * 2) {
      return { tooLarge: true, parts: [] as ReturnType<typeof diffLines> };
    }
    return { tooLarge: false, parts: diffLines(disk, mine) };
  });

  async function reload() {
    if (!conflict.tab) return;
    workspace.notifyExternalChange(conflict.tab.path, "modify");
    closeConflict();
  }

  async function keepMine() {
    const tab = conflict.tab;
    if (!tab) return;
    const my = conflict.myContent;
    closeConflict();
    try {
      const result = await writeTextFile(tab.path, my, undefined);
      workspace.patchTab(tab.id, {
        isDirty: false,
        lastKnownMtime: result.mtime,
      });
    } catch (e) {
      showError(
        `Failed to overwrite ${tab.title}: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }

  function toggleDiff() {
    conflict.showDiff = !conflict.showDiff;
  }
</script>

{#if conflict.isOpen && conflict.tab}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Conflict">
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={closeConflict}
      aria-label="Close"
    ></button>
    <div
      class="modal-box bg-base-100 border border-base-300 shadow-2xl flex flex-col p-0 overflow-hidden {conflict.showDiff
        ? 'max-w-3xl max-h-[80vh]'
        : 'max-w-md'}"
    >
      <div class="px-6 py-4 border-b border-base-200">
        <h3 class="font-bold text-lg">{conflict.tab.title} 在磁碟上被改過了</h3>
        <p class="text-sm text-base-content/60 mt-1">
          這個檔案被別的程式或外部編輯器改了。要怎麼處理你目前的編輯？
        </p>
      </div>

      {#if conflict.showDiff}
        <div class="flex-1 overflow-auto px-6 py-4 bg-base-200/40 font-mono text-xs">
          {#if conflict.diskContent === null}
            <div class="text-base-content/40">無法讀取磁碟版本</div>
          {:else if diffParts?.tooLarge}
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

      <div
        class="px-6 py-4 bg-base-200/50 border-t border-base-200 flex justify-between items-center gap-3"
      >
        <button
          type="button"
          class="btn btn-ghost btn-sm"
          onclick={toggleDiff}
        >
          {conflict.showDiff ? "隱藏 diff" : "看 diff"}
        </button>
        <div class="flex gap-2">
          <button
            type="button"
            class="btn btn-sm btn-ghost"
            onclick={reload}
          >
            重載並丟棄我的
          </button>
          <button
            type="button"
            class="btn btn-sm btn-primary"
            onclick={keepMine}
          >
            保留我的（覆寫）
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
