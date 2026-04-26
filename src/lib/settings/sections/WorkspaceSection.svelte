<script lang="ts">
  import {
    workspaceSettings,
    setAttachmentFolder,
    isValidFolder,
  } from "../workspaceSettings.svelte";

  let draft = $state(workspaceSettings.attachmentFolder);
  let error = $state<string | null>(null);

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
</style>
