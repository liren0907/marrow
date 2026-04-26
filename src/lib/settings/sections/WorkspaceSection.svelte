<script lang="ts">
  import {
    workspaceSettings,
    setAttachmentFolder,
    isValidFolder,
  } from "../workspaceSettings.svelte";
  import {
    serverSettings,
    addDenyEntry,
    removeDenyEntry,
    resetServerSettings,
    isValidDenyEntry,
    SERVER_DEFAULTS,
  } from "../serverSettings.svelte";

  let draft = $state(workspaceSettings.attachmentFolder);
  let error = $state<string | null>(null);
  let denyDraft = $state("");
  let denyError = $state<string | null>(null);

  // Keep draft in sync if the value changes via another surface.
  $effect(() => {
    if (workspaceSettings.attachmentFolder !== draft) {
      draft = workspaceSettings.attachmentFolder;
    }
  });

  function onCommit() {
    const trimmed = draft.trim();
    if (trimmed === workspaceSettings.attachmentFolder) {
      error = null;
      return;
    }
    if (!isValidFolder(trimmed)) {
      error =
        "Folder name must contain only letters, digits, dots, dashes, underscores, or slashes (no leading slash, no `..`).";
      return;
    }
    setAttachmentFolder(trimmed);
    error = null;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      onCommit();
    }
  }

  function onAddDeny() {
    const trimmed = denyDraft.trim();
    if (!trimmed) return;
    if (!isValidDenyEntry(trimmed)) {
      denyError =
        "Folder name must contain only letters, digits, dots, dashes, underscores. No slashes.";
      return;
    }
    if (serverSettings.denyList.includes(trimmed)) {
      denyError = "Already in the list.";
      return;
    }
    if (addDenyEntry(trimmed)) {
      denyDraft = "";
      denyError = null;
    }
  }

  function onDenyKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      onAddDeny();
    }
  }

  function onResetDeny() {
    if (
      confirm(
        "Restore the deny list to factory defaults? Custom entries will be lost.",
      )
    ) {
      resetServerSettings();
    }
  }

  // Common entries we warn about removing — these are the things most
  // workspaces really should hide. Removing intentionally is fine but
  // we want a moment of pause.
  const PROTECTIVE = new Set(SERVER_DEFAULTS.denyList);

  function onRemoveDeny(name: string) {
    if (PROTECTIVE.has(name)) {
      if (
        !confirm(
          `Remove "${name}" from the deny list?\n\nThis folder will then appear in the file tree, search results, and backlink index. This is rarely what you want.`,
        )
      ) {
        return;
      }
    }
    removeDenyEntry(name);
  }
</script>

<div class="section">
  <h3 class="section-title">Attachment folder</h3>
  <p class="section-desc">
    Pasted images are saved here, relative to the workspace root. Changing
    this only affects new pastes — existing files are not moved.
  </p>
  <div class="row">
    <input
      type="text"
      class="text-input"
      bind:value={draft}
      onblur={onCommit}
      onkeydown={onKeydown}
      placeholder="attachments"
      spellcheck="false"
      autocomplete="off"
    />
    <span class="hint">workspace/<strong>{draft || "?"}</strong>/</span>
  </div>
  {#if error}
    <div class="error">{error}</div>
  {/if}
</div>

<div class="section">
  <h3 class="section-title">Excluded folders</h3>
  <p class="section-desc">
    Folder basenames hidden from the file tree, search index, and backlink
    walker. Matched as exact basename (no globs, no nested paths). Affects
    every workspace.
  </p>

  {#if serverSettings.denyList.length === 0}
    <div class="empty-deny">
      No exclusions — every folder will be indexed and searched.
    </div>
  {:else}
    <div class="chips">
      {#each serverSettings.denyList as entry (entry)}
        <span class="chip" class:protective={PROTECTIVE.has(entry)}>
          <span class="chip-name">{entry}</span>
          <button
            class="chip-x"
            type="button"
            onclick={() => onRemoveDeny(entry)}
            aria-label={`Remove ${entry}`}
            title={PROTECTIVE.has(entry) ? "Default entry — confirm before removing" : "Remove"}
          >
            ×
          </button>
        </span>
      {/each}
    </div>
  {/if}

  <div class="add-row">
    <input
      type="text"
      class="text-input"
      bind:value={denyDraft}
      onkeydown={onDenyKeydown}
      placeholder="add folder name…"
      spellcheck="false"
      autocomplete="off"
    />
    <button class="btn-primary" type="button" onclick={onAddDeny}>Add</button>
  </div>
  {#if denyError}
    <div class="error">{denyError}</div>
  {/if}

  <div class="reset-row">
    <button class="btn-secondary" type="button" onclick={onResetDeny}>
      Reset to defaults
    </button>
  </div>
</div>

<style>
  .section {
    margin-bottom: 28px;
  }
  .section-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-base-content);
    margin: 0 0 4px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--mw-ink-2);
    margin: 0 0 12px;
    line-height: 1.5;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .text-input {
    flex: 0 0 240px;
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 6px 10px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--color-base-content);
  }
  .text-input:focus {
    outline: none;
    border-color: var(--mw-accent);
  }
  .hint {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--mw-ink-3);
  }
  .hint strong {
    color: var(--color-base-content);
    font-weight: 500;
  }
  .error {
    margin-top: 8px;
    font-size: 11px;
    color: oklch(0.6 0.18 25);
  }
  .empty-deny {
    padding: 10px 12px;
    background: var(--color-base-200);
    border: 1px dashed var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    font-size: 12px;
    color: var(--mw-ink-3);
    font-style: italic;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 12px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 3px 4px 3px 8px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--color-base-content);
  }
  .chip.protective {
    border-color: color-mix(in oklch, var(--mw-accent) 40%, var(--mw-rule));
  }
  .chip-name {
    line-height: 1;
  }
  .chip-x {
    background: transparent;
    border: none;
    color: var(--mw-ink-3);
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 1px 4px;
    border-radius: var(--mw-radius-xs);
  }
  .chip-x:hover {
    background: var(--color-base-300);
    color: oklch(0.6 0.18 25);
  }
  .add-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .add-row .text-input {
    flex: 1;
    max-width: 240px;
  }
  .btn-primary {
    background: var(--mw-accent);
    color: var(--color-base-100);
    border: 1px solid var(--mw-accent);
    border-radius: var(--mw-radius-sm);
    padding: 6px 14px;
    font-size: 12px;
    cursor: pointer;
    font-weight: 500;
  }
  .btn-primary:hover {
    filter: brightness(1.1);
  }
  .reset-row {
    margin-top: 14px;
  }
  .btn-secondary {
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 6px 12px;
    font-size: 12px;
    color: var(--mw-ink-2);
    cursor: pointer;
  }
  .btn-secondary:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
</style>
