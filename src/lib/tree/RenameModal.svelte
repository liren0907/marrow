<script lang="ts">
  import { renameModal, closeRenameModal } from "./renameModalState.svelte";
  import { previewRename, executeRename } from "./renameRefactor";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { showError, showSuccess } from "$lib/stores/toastStore.svelte";

  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (renameModal.isOpen && renameModal.stage === "name") {
      queueMicrotask(() => {
        inputEl?.focus();
        // Select the basename portion (without .md) so the user types over it.
        if (inputEl) {
          const v = inputEl.value;
          const dotIdx = v.lastIndexOf(".");
          if (dotIdx > 0) {
            inputEl.setSelectionRange(0, dotIdx);
          } else {
            inputEl.select();
          }
        }
      });
    }
  });

  function relPath(path: string): string {
    const root = workspace.info?.root ?? "";
    if (root && path.startsWith(root)) {
      return path.slice(root.length).replace(/^[/\\]/, "");
    }
    return path;
  }

  function buildPreview() {
    if (!renameModal.newName.trim()) return;
    try {
      renameModal.preview = previewRename(
        renameModal.oldPath,
        renameModal.newName,
      );
      renameModal.stage = "preview";
    } catch (e) {
      showError(`Preview failed: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  async function confirm() {
    if (!renameModal.preview) return;
    const preview = renameModal.preview;
    renameModal.stage = "running";
    const { failures } = await executeRename(preview);
    closeRenameModal();
    if (failures.length === 0) {
      showSuccess(
        `Renamed to ${preview.newBasename}.md (updated ${preview.affectedFiles.length - 1} files)`,
      );
    } else {
      const list = failures
        .slice(0, 3)
        .map((f) => relPath(f.path))
        .join(", ");
      const more = failures.length > 3 ? ` (+${failures.length - 3} more)` : "";
      showError(`Rename completed with ${failures.length} failures: ${list}${more}`);
    }
  }

  function back() {
    renameModal.stage = "name";
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (renameModal.stage === "name") buildPreview();
      else if (renameModal.stage === "preview") void confirm();
    }
  }
</script>

{#if renameModal.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Rename file">
    <button
      type="button"
      class="modal-backdrop cursor-default"
      onclick={closeRenameModal}
      aria-label="Close"
    ></button>
    <div
      class="modal-box bg-base-100 border border-base-300 max-w-lg p-0 max-h-[80vh] flex flex-col"
    >
      <div class="px-6 py-4 border-b border-base-200">
        <h3 class="font-bold text-lg">
          {#if renameModal.stage === "running"}
            Renaming…
          {:else}
            Rename file
          {/if}
        </h3>
        <p class="text-xs text-base-content/50 truncate mt-0.5">
          {relPath(renameModal.oldPath)}
        </p>
      </div>

      <div class="flex-1 overflow-y-auto px-6 py-4">
        {#if renameModal.stage === "name"}
          <label for="rename-new-name" class="block text-xs text-base-content/60 mb-1">
            New filename
          </label>
          <input
            id="rename-new-name"
            bind:this={inputEl}
            type="text"
            class="input input-bordered w-full"
            bind:value={renameModal.newName}
            onkeydown={onKey}
          />
          <p class="text-xs text-base-content/40 mt-2">
            Wiki-link references in other markdown files will be updated to
            match the new name.
          </p>
        {:else if renameModal.stage === "preview" && renameModal.preview}
          {@const p = renameModal.preview}
          <p class="text-sm">
            Will rename
            <code class="px-1 bg-base-200 rounded">{p.oldBasename}.md</code>
            →
            <code class="px-1 bg-base-200 rounded">{p.newBasename}.md</code>
          </p>
          {#if p.affectedFiles.length > 1}
            <p class="text-sm mt-3 text-base-content/70">
              Will update wiki-link references in
              <strong>{p.affectedFiles.length - 1}</strong> other file{p.affectedFiles.length - 1 === 1 ? "" : "s"}:
            </p>
            <ul class="mt-2 text-xs font-mono text-base-content/60 max-h-48 overflow-y-auto border border-base-200 rounded p-2">
              {#each p.affectedFiles as path}
                {#if path !== p.oldPath}
                  <li class="truncate py-0.5">{relPath(path)}</li>
                {/if}
              {/each}
            </ul>
          {:else}
            <p class="text-sm mt-3 text-base-content/50 italic">
              No other files reference this file.
            </p>
          {/if}
        {:else if renameModal.stage === "running"}
          <p class="text-sm text-base-content/60">Working…</p>
        {/if}
      </div>

      <div class="px-6 py-4 bg-base-200/50 border-t border-base-200 flex justify-end gap-2">
        {#if renameModal.stage === "name"}
          <button type="button" class="btn btn-ghost btn-sm" onclick={closeRenameModal}>
            Cancel
          </button>
          <button
            type="button"
            class="btn btn-primary btn-sm"
            onclick={buildPreview}
            disabled={!renameModal.newName.trim()}
          >
            Next
          </button>
        {:else if renameModal.stage === "preview"}
          <button type="button" class="btn btn-ghost btn-sm" onclick={back}>
            Back
          </button>
          <button type="button" class="btn btn-primary btn-sm" onclick={confirm}>
            Confirm rename
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
