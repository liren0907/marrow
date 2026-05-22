<script lang="ts">
  import {
    accent,
    setAccent,
    accentColor,
    ACCENTS,
    type AccentKey,
  } from "./accentState.svelte";
  import { tweaks, closeTweaks } from "./tweaksState.svelte";
  import {
    appearance,
    setEditorFont,
    setEditorFontSize,
    setEditorLineHeight,
    setEditorWidth,
    EDITOR_FONT_LABELS,
    EDITOR_FONT_STACKS,
    FONT_SIZE_MIN,
    FONT_SIZE_MAX,
    LINE_HEIGHT_MIN,
    LINE_HEIGHT_MAX,
    EDITOR_WIDTH_MIN,
    EDITOR_WIDTH_MAX,
    formatEditorWidth,
    type EditorFontKey,
  } from "./appearanceSettings.svelte";
  import { uiSettings, togglePaneOutline } from "./uiSettings.svelte";
  import { openSettings } from "./settingsModalState.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  const accentEntries: { key: AccentKey; label: string }[] = (
    Object.keys(ACCENTS) as AccentKey[]
  ).map((k) => ({ key: k, label: ACCENTS[k].label }));

  const fontEntries = Object.entries(EDITOR_FONT_LABELS) as [
    EditorFontKey,
    string,
  ][];

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

  function onFontChange(e: Event) {
    setEditorFont((e.currentTarget as HTMLSelectElement).value as EditorFontKey);
  }
  function onFontSizeInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setEditorFontSize(v);
  }
  function onLineHeightInput(e: Event) {
    const v = parseFloat((e.target as HTMLInputElement).value);
    if (Number.isFinite(v)) setEditorLineHeight(v);
  }
  function onWidthInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setEditorWidth(v);
  }

  function openFullSettings() {
    closeTweaks();
    openSettings();
  }

  let panelEl: HTMLDivElement | undefined = $state();

  function onWindowMouseDown(e: MouseEvent) {
    if (!tweaks.isOpen) return;
    const target = e.target as Node;
    if (panelEl?.contains(target)) return;
    // Skip clicks on any tweaks trigger button — they call toggleTweaks()
    // themselves, and treating them as outside-clicks would close-then-reopen
    // (or vice versa) leading to flicker / no-op.
    if (
      target instanceof Element &&
      target.closest("[data-tweaks-trigger]")
    ) {
      return;
    }
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

    <div class="tweaks-section">
      <div class="tweaks-label">Editor font</div>
      <select
        class="tweaks-select"
        value={appearance.editorFont}
        onchange={onFontChange}
      >
        {#each fontEntries as [key, label] (key)}
          <option value={key} style:font-family={EDITOR_FONT_STACKS[key]}>
            {label}
          </option>
        {/each}
      </select>
    </div>

    <div class="tweaks-section">
      <div class="tweaks-row-head">
        <span class="tweaks-label">Font size</span>
        <span class="tweaks-value">{appearance.editorFontSize} px</span>
      </div>
      <input
        class="tweaks-range"
        type="range"
        min={FONT_SIZE_MIN}
        max={FONT_SIZE_MAX}
        step="1"
        value={appearance.editorFontSize}
        oninput={onFontSizeInput}
      />
    </div>

    <div class="tweaks-section">
      <div class="tweaks-row-head">
        <span class="tweaks-label">Line height</span>
        <span class="tweaks-value">{appearance.editorLineHeight.toFixed(2)}</span>
      </div>
      <input
        class="tweaks-range"
        type="range"
        min={LINE_HEIGHT_MIN}
        max={LINE_HEIGHT_MAX}
        step="0.05"
        value={appearance.editorLineHeight}
        oninput={onLineHeightInput}
      />
    </div>

    <div class="tweaks-section">
      <div class="tweaks-row-head">
        <span class="tweaks-label">Content width</span>
        <span class="tweaks-value">{formatEditorWidth(appearance.editorWidth)}</span>
      </div>
      <input
        class="tweaks-range"
        type="range"
        min={EDITOR_WIDTH_MIN}
        max={EDITOR_WIDTH_MAX}
        step="2"
        value={appearance.editorWidth}
        oninput={onWidthInput}
      />
    </div>

    <div class="tweaks-section">
      <label class="tweaks-check">
        <input
          type="checkbox"
          checked={uiSettings.showPaneOutline}
          onchange={togglePaneOutline}
        />
        <span>Show document outline</span>
      </label>
    </div>

    <button class="tweaks-more" onclick={openFullSettings}>
      <Icon name="settings" size={12} />
      <span>Open all settings</span>
      <span class="tweaks-more-arrow">→</span>
    </button>

    <div class="tweaks-hint">⌘P Quick Open · ⇧⌘P Command · ⌘, Tweaks</div>
  </div>
{/if}

<style>
  .tweaks-panel {
    position: fixed;
    /* Anchored top-right to align with the Pane gear trigger that opens
       this panel. Sits below the titlebar + tab row with a small gap. */
    top: calc(var(--mw-titlebar-h) + var(--mw-tab-h) + 6px);
    right: 16px;
    width: 256px;
    max-height: calc(100vh - var(--mw-titlebar-h) - var(--mw-statusbar-h) - 32px);
    overflow-y: auto;
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
  .tweaks-row-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 4px;
  }
  .tweaks-label {
    font-size: 11px;
    color: var(--mw-ink-2);
    margin-bottom: 6px;
  }
  .tweaks-value {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-base-content);
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
  .tweaks-select {
    width: 100%;
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 4px 8px;
    font-size: 11.5px;
    color: var(--color-base-content);
    cursor: pointer;
  }
  .tweaks-select:focus {
    outline: none;
    border-color: var(--mw-accent);
  }
  .tweaks-range {
    width: 100%;
    accent-color: var(--mw-accent);
  }
  .tweaks-check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--color-base-content);
    cursor: pointer;
  }
  .tweaks-check input {
    accent-color: var(--mw-accent);
  }
  .tweaks-more {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 6px 10px;
    margin-top: 4px;
    font-size: 11.5px;
    color: var(--mw-ink-2);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .tweaks-more:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
  .tweaks-more-arrow {
    margin-left: auto;
    color: var(--mw-ink-3);
  }
  .tweaks-hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--mw-ink-3);
    margin-top: 10px;
    line-height: 1.5;
  }
</style>
