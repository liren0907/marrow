<script lang="ts">
  import {
    accent,
    setAccent,
    accentColor,
    ACCENTS,
    type AccentKey,
  } from "./accentState.svelte";
  import { tweaks, closeTweaks } from "./tweaksState.svelte";

  const accentEntries: { key: AccentKey; label: string }[] = (
    Object.keys(ACCENTS) as AccentKey[]
  ).map((k) => ({ key: k, label: ACCENTS[k].label }));

  // Three modes mirror AppearanceSection. light → marrow-light (Notion-y soft
  // white), intermediate → marrow-pro-light (cream paper, Marrow's signature),
  // dark → marrow-pro-dark.
  type ThemeMode = "light" | "intermediate" | "dark";
  let theme = $state<ThemeMode>("intermediate");

  $effect(() => {
    if (!tweaks.isOpen) return;
    const t = document.documentElement.getAttribute("data-theme") ?? "";
    if (t === "marrow-pro-dark" || t === "dark") theme = "dark";
    else if (t === "marrow-light") theme = "light";
    else theme = "intermediate";
  });

  function setTheme(next: ThemeMode) {
    theme = next;
    const name =
      next === "dark"
        ? "marrow-pro-dark"
        : next === "light"
          ? "marrow-light"
          : "marrow-pro-light";
    document.documentElement.setAttribute("data-theme", name);
    localStorage.setItem("theme", name);
  }

  // Accent preview only needs light-vs-dark contrast — light & intermediate
  // share the same accent palette.
  const accentMode = $derived<"light" | "dark">(
    theme === "dark" ? "dark" : "light",
  );

  let panelEl: HTMLDivElement | undefined = $state();

  function onWindowMouseDown(e: MouseEvent) {
    if (!tweaks.isOpen) return;
    const target = e.target as Node;
    if (panelEl?.contains(target)) return;
    closeTweaks();
  }
  function onWindowKeyDown(e: KeyboardEvent) {
    if (tweaks.isOpen && e.key === "Escape") closeTweaks();
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={onWindowKeyDown} />

{#if tweaks.isOpen}
  <div bind:this={panelEl} class="tweaks-panel" role="dialog" aria-label="Tweaks">
    <div class="tweaks-head">
      <span class="tweaks-title mw-meta">Tweaks</span>
      <button class="tweaks-close" onclick={closeTweaks} aria-label="Close">×</button>
    </div>

    <div class="tweaks-section">
      <div class="tweaks-label">Theme</div>
      <div class="tweaks-seg">
        <button class:on={theme === "light"} onclick={() => setTheme("light")}>
          Light
        </button>
        <button
          class:on={theme === "intermediate"}
          onclick={() => setTheme("intermediate")}
          title="Marrow's signature warm cream"
        >
          Inter.
        </button>
        <button class:on={theme === "dark"} onclick={() => setTheme("dark")}>
          Dark
        </button>
      </div>
    </div>

    <div class="tweaks-section">
      <div class="tweaks-label">Accent</div>
      <div class="tweaks-swatches">
        {#each accentEntries as entry (entry.key)}
          <button
            class="tweaks-swatch"
            class:on={accent.current === entry.key}
            onclick={() => setAccent(entry.key)}
            title={entry.label}
            aria-label={entry.label}
            style:background={accentColor(entry.key, accentMode)}
          ></button>
        {/each}
      </div>
    </div>

    <div class="tweaks-hint">⌘P Quick Open · ⇧⌘P Command · ⌘, Tweaks</div>
  </div>
{/if}

<style>
  .tweaks-panel {
    position: fixed;
    bottom: calc(var(--mw-statusbar-h) + 16px);
    right: 20px;
    width: 240px;
    background: var(--mw-bg-elev);
    border: 1px solid var(--mw-rule-strong);
    border-radius: var(--mw-radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
    padding: 14px;
    z-index: 60;
    font-size: 12px;
    color: var(--color-base-content);
  }
  .tweaks-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .tweaks-close {
    color: var(--mw-ink-3);
    font-size: 18px;
    line-height: 1;
    padding: 0 4px;
    background: transparent;
    border: none;
    cursor: pointer;
  }
  .tweaks-close:hover {
    color: var(--color-base-content);
  }
  .tweaks-section {
    margin-bottom: 14px;
  }
  .tweaks-label {
    font-size: 11px;
    color: var(--mw-ink-2);
    margin-bottom: 6px;
  }
  .tweaks-seg {
    display: flex;
    gap: 2px;
    background: var(--color-base-300);
    padding: 2px;
    border-radius: var(--mw-radius-sm);
  }
  .tweaks-seg button {
    flex: 1;
    background: transparent;
    border: none;
    padding: 4px 0;
    font-size: 11px;
    color: var(--mw-ink-2);
    cursor: pointer;
    border-radius: var(--mw-radius-xs);
  }
  .tweaks-seg button.on {
    background: var(--color-base-100);
    color: var(--color-base-content);
  }
  .tweaks-swatches {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 6px;
  }
  .tweaks-swatch {
    aspect-ratio: 1;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s;
  }
  .tweaks-swatch:hover {
    transform: scale(1.1);
  }
  .tweaks-swatch.on {
    border-color: var(--color-base-content);
  }
  .tweaks-hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--mw-ink-3);
    margin-top: 8px;
    line-height: 1.5;
  }
</style>
