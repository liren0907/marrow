<script lang="ts">
  import { commandPalette } from "./commandPaletteState.svelte";
  import { getCommands, type Command } from "./commands";
  import { fuzzyMatch } from "$lib/quickopen/fuzzy";

  interface Result {
    cmd: Command;
    score: number;
    indices: number[];
  }

  let inputEl: HTMLInputElement | undefined = $state();

  const results = $derived.by(() => {
    const q = commandPalette.query;
    const all = getCommands();
    if (!q) {
      return all.map(
        (cmd) => ({ cmd, score: 0, indices: [] }) satisfies Result,
      );
    }
    const out: Result[] = [];
    for (const cmd of all) {
      const m = fuzzyMatch(q, cmd.title);
      if (!m) continue;
      out.push({ cmd, score: m.score, indices: m.indices });
    }
    out.sort(
      (a, b) => b.score - a.score || a.cmd.title.localeCompare(b.cmd.title),
    );
    return out;
  });

  $effect(() => {
    if (commandPalette.isOpen) {
      if (commandPalette.selectedIdx >= results.length)
        commandPalette.selectedIdx = 0;
      queueMicrotask(() => inputEl?.focus());
    }
  });

  function close() {
    commandPalette.isOpen = false;
  }

  function pick(idx: number) {
    const r = results[idx];
    if (!r) return;
    // Close BEFORE executing so the action can open another modal/panel
    // without racing against our own state.
    close();
    try {
      r.cmd.action();
    } catch (e) {
      console.error("[command] action failed", e);
    }
  }

  function onInputKeydown(e: KeyboardEvent) {
    if (results.length === 0) {
      if (e.key === "Escape") close();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      commandPalette.selectedIdx =
        (commandPalette.selectedIdx + 1) % results.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      commandPalette.selectedIdx =
        (commandPalette.selectedIdx - 1 + results.length) % results.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      pick(commandPalette.selectedIdx);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
  }

  function highlight(
    title: string,
    indices: number[],
  ): { ch: string; on: boolean }[] {
    const set = new Set(indices);
    const out: { ch: string; on: boolean }[] = [];
    for (let i = 0; i < title.length; i++) {
      out.push({ ch: title[i], on: set.has(i) });
    }
    return out;
  }
</script>

{#if commandPalette.isOpen}
  <div class="modal modal-open z-[60]" role="dialog" aria-label="Command palette">
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
          placeholder="Type a command…"
          bind:value={commandPalette.query}
          onkeydown={onInputKeydown}
        />
      </div>
      <ul class="flex-1 overflow-y-auto py-1">
        {#if results.length === 0}
          <li class="px-4 py-3 text-sm text-base-content/40 italic">
            No matches
          </li>
        {:else}
          {#each results as r, i (r.cmd.id)}
            <li>
              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-1.5 text-left text-sm hover:bg-base-200"
                class:bg-base-200={i === commandPalette.selectedIdx}
                onmousemove={() => (commandPalette.selectedIdx = i)}
                onclick={() => pick(i)}
              >
                <span
                  class="text-[10px] uppercase tracking-wide text-base-content/40 shrink-0 w-16"
                  >{r.cmd.category}</span
                >
                <span class="truncate flex-1">
                  {#each highlight(r.cmd.title, r.indices) as part}
                    {#if part.on}
                      <span class="text-primary font-semibold">{part.ch}</span>
                    {:else}
                      {part.ch}
                    {/if}
                  {/each}
                </span>
                {#if r.cmd.shortcut}
                  <span
                    class="text-[11px] font-mono text-base-content/40 shrink-0"
                    >{r.cmd.shortcut}</span
                  >
                {/if}
              </button>
            </li>
          {/each}
        {/if}
      </ul>
      <div
        class="px-3 py-1.5 border-t border-base-200 text-[10px] text-base-content/40 flex gap-3"
      >
        <span>↑↓ select</span>
        <span>Enter run</span>
        <span>Esc close</span>
      </div>
    </div>
  </div>
{/if}
