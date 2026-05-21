<script lang="ts">
  // Settings → Keyboard. Lists every action from the registry grouped by
  // category and lets the user rebind each one. Recording captures the next
  // keystroke globally; the keyboard matcher is silenced meanwhile via
  // recordingState.isRecording.
  import { onDestroy } from "svelte";
  import {
    getActions,
    eventToCombo,
    comboToDisplay,
    isModifierCode,
    RESERVED,
    type ActionDef,
    type ActionCategory,
  } from "$lib/command/actions";
  import {
    keybindingOverrides,
    recordingState,
    resolveBinding,
    setBinding,
    resetBinding,
    resetAll,
  } from "$lib/settings/keybindingSettings.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  const CATEGORY_ORDER: ActionCategory[] = [
    "Navigation",
    "View",
    "Workspace",
    "Editor",
  ];

  let filter = $state("");
  let recordingId = $state<string | null>(null);
  let recordError = $state("");

  const actions = $derived(getActions());

  // combo → ids currently bound to it (for conflict detection).
  const comboOwners = $derived.by(() => {
    const map = new Map<string, string[]>();
    for (const a of actions) {
      for (const combo of resolveBinding(a.id)) {
        const list = map.get(combo);
        if (list) list.push(a.id);
        else map.set(combo, [a.id]);
      }
    }
    return map;
  });

  const grouped = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    const map = new Map<ActionCategory, ActionDef[]>();
    for (const a of actions) {
      if (q && !a.title.toLowerCase().includes(q)) continue;
      const list = map.get(a.category);
      if (list) list.push(a);
      else map.set(a.category, [a]);
    }
    return map;
  });

  const hasOverrides = $derived(Object.keys(keybindingOverrides).length > 0);

  function titleOf(id: string): string {
    return actions.find((a) => a.id === id)?.title ?? id;
  }

  function startRecord(id: string): void {
    recordingId = id;
    recordError = "";
    recordingState.isRecording = true;
  }

  function cancelRecord(): void {
    recordingId = null;
    recordError = "";
    recordingState.isRecording = false;
  }

  // While a row is recording, capture keystrokes globally. The matcher in
  // shortcuts.svelte.ts is already silenced via recordingState.isRecording.
  // A pointerdown anywhere cancels — covers clicking away / switching tabs.
  $effect(() => {
    if (recordingId === null) return;

    function onKey(e: KeyboardEvent) {
      const id = recordingId;
      if (id === null) return;
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        cancelRecord();
        return;
      }
      if (isModifierCode(e.code)) return; // wait for a non-modifier key
      const combo = eventToCombo(e);
      if (!combo) {
        recordError = "Shortcut must include ⌘ or Ctrl";
        return;
      }
      if (RESERVED[combo]) {
        recordError = `Reserved for ${RESERVED[combo]}`;
        return;
      }
      const conflict = (comboOwners.get(combo) ?? []).find(
        (oid) => oid !== id,
      );
      if (conflict) {
        recordError = `Already bound to "${titleOf(conflict)}"`;
        return;
      }
      setBinding(id, [combo]);
      cancelRecord();
    }
    function onPointerDown() {
      cancelRecord();
    }
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("pointerdown", onPointerDown, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("pointerdown", onPointerDown, true);
    };
  });

  // Safety net: if the Settings tab is closed mid-recording, clear the flag
  // so global shortcuts don't stay disabled.
  onDestroy(() => {
    recordingState.isRecording = false;
  });
</script>

<div class="section">
  <h3 class="section-title">Keyboard shortcuts</h3>
  <p class="section-desc">
    Click a shortcut to rebind it. Every shortcut must include ⌘/Ctrl so it
    never interferes with typing. While recording, press Esc — or click
    anywhere — to cancel.
  </p>
  <div class="kb-toolbar">
    <input
      type="text"
      class="kb-filter"
      placeholder="Filter shortcuts…"
      bind:value={filter}
    />
    {#if hasOverrides}
      <button type="button" class="kb-reset-all" onclick={resetAll}>
        Reset all to defaults
      </button>
    {/if}
  </div>
</div>

{#each CATEGORY_ORDER as cat (cat)}
  {@const list = grouped.get(cat) ?? []}
  {#if list.length}
    <div class="kb-group">
      <div class="kb-group-label">{cat}</div>
      {#each list as a (a.id)}
        {@const binds = resolveBinding(a.id)}
        {@const isRec = recordingId === a.id}
        {@const overridden = !!keybindingOverrides[a.id]}
        <div class="kb-row" class:recording={isRec}>
          <span class="kb-title">{a.title}</span>
          {#if isRec}
            <span class="kb-hint">Press keys… · Esc to cancel</span>
          {:else}
            <button
              type="button"
              class="kb-binding"
              onclick={() => startRecord(a.id)}
              aria-label="Rebind {a.title}"
            >
              {#if binds.length}
                {#each binds as combo (combo)}
                  <kbd class="kb-key">{comboToDisplay(combo)}</kbd>
                {/each}
              {:else}
                <span class="kb-unset">Not set</span>
              {/if}
            </button>
          {/if}
          {#if overridden && !isRec}
            <button
              type="button"
              class="kb-icon-btn"
              onclick={() => resetBinding(a.id)}
              title="Reset to default"
              aria-label="Reset {a.title} to default"
            >
              <Icon name="rotate-ccw" size={13} />
            </button>
          {/if}
        </div>
        {#if isRec && recordError}
          <div class="kb-error">{recordError}</div>
        {/if}
      {/each}
    </div>
  {/if}
{/each}

<style>
  .section {
    margin-bottom: 24px;
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
    margin: 0 0 14px;
    line-height: 1.5;
  }

  .kb-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .kb-filter {
    flex: 1;
    font-size: 12.5px;
    padding: 5px 9px;
    border: 1px solid var(--color-base-300);
    border-radius: var(--mw-radius-xs);
    background: var(--color-base-100);
    color: var(--color-base-content);
    outline: none;
  }
  .kb-filter:focus {
    border-color: var(--mw-accent);
  }
  .kb-reset-all {
    font-size: 12px;
    padding: 5px 10px;
    border: 1px solid var(--color-base-300);
    border-radius: var(--mw-radius-xs);
    background: var(--color-base-100);
    color: var(--mw-ink-2);
    cursor: pointer;
    white-space: nowrap;
  }
  .kb-reset-all:hover {
    background: var(--color-base-200);
    color: var(--color-base-content);
  }

  .kb-group {
    margin-bottom: 18px;
  }
  .kb-group-label {
    font-size: 10.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--mw-ink-2);
    margin: 0 0 4px;
    padding: 0 8px;
  }

  .kb-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px;
    border-radius: var(--mw-radius-xs);
    min-height: 32px;
  }
  .kb-row:hover {
    background: var(--color-base-200);
  }
  .kb-row.recording {
    background: color-mix(in oklch, var(--mw-accent) 12%, transparent);
  }

  .kb-title {
    flex: 1;
    font-size: 12.5px;
    color: var(--color-base-content);
  }

  .kb-binding {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px;
    border: 1px solid transparent;
    border-radius: var(--mw-radius-xs);
    background: transparent;
    cursor: pointer;
  }
  .kb-binding:hover {
    border-color: var(--color-base-300);
    background: var(--color-base-100);
  }

  .kb-key {
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.4;
    padding: 1px 6px;
    background: var(--color-base-300);
    color: var(--color-base-content);
    border-radius: 3px;
  }
  .kb-unset {
    font-size: 11.5px;
    font-style: italic;
    color: var(--mw-ink-2);
  }
  .kb-hint {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--mw-accent);
  }

  .kb-icon-btn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--mw-ink-2);
    border-radius: var(--mw-radius-xs);
    cursor: pointer;
  }
  .kb-icon-btn:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }

  .kb-error {
    font-size: 11px;
    color: var(--color-error);
    padding: 2px 8px 6px;
  }
</style>
