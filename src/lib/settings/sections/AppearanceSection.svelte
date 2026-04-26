<script lang="ts">
  import {
    accent,
    setAccent,
    accentColor,
    ACCENTS,
    type AccentKey,
  } from "../accentState.svelte";
  import {
    uiSettings,
    toggleBreadcrumb,
    togglePaneOutline,
  } from "../uiSettings.svelte";
  import {
    appearance,
    setCmTheme,
    setPrismTheme,
    setEditorFont,
    setEditorFontSize,
    CM_THEME_LABELS,
    PRISM_THEME_LABELS,
    EDITOR_FONT_LABELS,
    EDITOR_FONT_STACKS,
    FONT_SIZE_MIN,
    FONT_SIZE_MAX,
    type CodeMirrorThemeKey,
    type PrismThemeKey,
    type EditorFontKey,
  } from "../appearanceSettings.svelte";

  const accentEntries: { key: AccentKey; label: string }[] = (
    Object.keys(ACCENTS) as AccentKey[]
  ).map((k) => ({ key: k, label: ACCENTS[k].label }));

  const cmThemeEntries = Object.entries(CM_THEME_LABELS) as [
    CodeMirrorThemeKey,
    string,
  ][];
  const prismThemeEntries = Object.entries(PRISM_THEME_LABELS) as [
    PrismThemeKey,
    string,
  ][];
  const fontEntries = Object.entries(EDITOR_FONT_LABELS) as [
    EditorFontKey,
    string,
  ][];

  function onFontSizeInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setEditorFontSize(v);
  }
  function onEditorFontChange(e: Event) {
    setEditorFont((e.currentTarget as HTMLSelectElement).value as EditorFontKey);
  }
  function onCmThemeChange(e: Event) {
    setCmTheme((e.currentTarget as HTMLSelectElement).value as CodeMirrorThemeKey);
  }
  function onPrismThemeChange(e: Event) {
    setPrismTheme((e.currentTarget as HTMLSelectElement).value as PrismThemeKey);
  }

  // Theme is stored in localStorage and reflected on <html data-theme>.
  // Mirrors the logic in TweaksPanel — kept duplicated rather than
  // extracted because the two surfaces stay decoupled.
  //
  // Three modes:
  //   light        → marrow-light       (Notion-inspired soft white)
  //   intermediate → marrow-pro-light   (cream paper — Marrow's signature)
  //   dark         → marrow-pro-dark
  type ThemeMode = "light" | "intermediate" | "dark";
  let theme = $state<ThemeMode>("intermediate");

  function readMode(): ThemeMode {
    const t = document.documentElement.getAttribute("data-theme") ?? "";
    if (t === "marrow-pro-dark" || t === "dark") return "dark";
    if (t === "marrow-light") return "light";
    return "intermediate";
  }

  $effect(() => {
    theme = readMode();
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

  // Accent swatch preview only cares about light-vs-dark contrast —
  // intermediate uses the same accent palette as light.
  const accentMode = $derived<"light" | "dark">(
    theme === "dark" ? "dark" : "light",
  );
</script>

<div class="section">
  <h3 class="section-title">Theme</h3>
  <p class="section-desc">
    <strong>Light</strong> is Notion-inspired soft white,
    <strong>Intermediate</strong> is Marrow's signature warm cream, and
    <strong>Dark</strong> is the low-light counterpart.
  </p>
  <div class="seg">
    <button class:on={theme === "light"} onclick={() => setTheme("light")}>
      Light
    </button>
    <button
      class:on={theme === "intermediate"}
      onclick={() => setTheme("intermediate")}
    >
      Intermediate
    </button>
    <button class:on={theme === "dark"} onclick={() => setTheme("dark")}>
      Dark
    </button>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Accent color</h3>
  <p class="section-desc">
    Used for the active tab indicator, links, and focus rings.
  </p>
  <div class="swatches">
    {#each accentEntries as entry (entry.key)}
      <button
        class="swatch"
        class:on={accent.current === entry.key}
        onclick={() => setAccent(entry.key)}
        title={entry.label}
        aria-label={entry.label}
        style:background={accentColor(entry.key, accentMode)}
      ></button>
    {/each}
  </div>
</div>

<div class="section">
  <h3 class="section-title">Interface</h3>
  <p class="section-desc">Toggle ambient chrome around the editor.</p>
  <label class="check">
    <input
      type="checkbox"
      checked={uiSettings.showBreadcrumb}
      onchange={toggleBreadcrumb}
    />
    <span>Show breadcrumb above each pane</span>
  </label>
  <label class="check">
    <input
      type="checkbox"
      checked={uiSettings.showPaneOutline}
      onchange={togglePaneOutline}
    />
    <span>Show document outline in pane header</span>
  </label>
</div>

<div class="section">
  <h3 class="section-title">Editor font</h3>
  <p class="section-desc">
    Font used for the prose body in the markdown editor. Headings keep
    using the chrome display font.
  </p>
  <select
    class="select-input"
    value={appearance.editorFont}
    onchange={onEditorFontChange}
  >
    {#each fontEntries as [key, label] (key)}
      <option value={key} style:font-family={EDITOR_FONT_STACKS[key]}>
        {label}
      </option>
    {/each}
  </select>
  <div class="font-preview" style:font-family={EDITOR_FONT_STACKS[appearance.editorFont]}>
    The quick brown fox jumps over the lazy dog. 1234567890
  </div>
</div>

<div class="section">
  <h3 class="section-title">Editor font size</h3>
  <p class="section-desc">Body text size in the markdown editor.</p>
  <div class="row">
    <input
      type="range"
      min={FONT_SIZE_MIN}
      max={FONT_SIZE_MAX}
      step="1"
      value={appearance.editorFontSize}
      oninput={onFontSizeInput}
    />
    <span class="value">{appearance.editorFontSize} px</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Code editor theme</h3>
  <p class="section-desc">
    Syntax highlighting for the read-only text viewer (.txt / .ts / .py /
    etc.). "Auto" tracks the Marrow light/dark theme.
  </p>
  <select
    class="select-input"
    value={appearance.cmTheme}
    onchange={onCmThemeChange}
  >
    {#each cmThemeEntries as [key, label] (key)}
      <option value={key}>{label}</option>
    {/each}
  </select>
</div>

<div class="section">
  <h3 class="section-title">Code block syntax colors</h3>
  <p class="section-desc">
    Coloring for fenced code blocks rendered inside transclusion previews
    and other markdown surfaces (Prism).
  </p>
  <select
    class="select-input"
    value={appearance.prismTheme}
    onchange={onPrismThemeChange}
  >
    {#each prismThemeEntries as [key, label] (key)}
      <option value={key}>{label}</option>
    {/each}
  </select>
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
  }
  .seg {
    display: inline-flex;
    background: var(--color-base-300);
    padding: 2px;
    border-radius: var(--mw-radius-sm);
    gap: 2px;
  }
  .seg button {
    background: transparent;
    border: none;
    padding: 6px 18px;
    font-size: 12px;
    color: var(--mw-ink-2);
    cursor: pointer;
    border-radius: var(--mw-radius-xs);
  }
  .seg button.on {
    background: var(--color-base-100);
    color: var(--color-base-content);
  }
  .swatches {
    display: flex;
    gap: 10px;
  }
  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s;
  }
  .swatch:hover {
    transform: scale(1.1);
  }
  .swatch.on {
    border-color: var(--color-base-content);
  }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--color-base-content);
    margin-bottom: 8px;
    cursor: pointer;
  }
  .check input {
    accent-color: var(--mw-accent);
  }
  .select-input {
    width: 100%;
    max-width: 320px;
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 6px 10px;
    font-size: 12.5px;
    color: var(--color-base-content);
    cursor: pointer;
  }
  .select-input:focus {
    outline: none;
    border-color: var(--mw-accent);
  }
  .font-preview {
    margin-top: 10px;
    padding: 10px 12px;
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    font-size: 14px;
    color: var(--color-base-content);
    line-height: 1.5;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .row input[type="range"] {
    flex: 1;
    accent-color: var(--mw-accent);
  }
  .value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-base-content);
    min-width: 64px;
    text-align: right;
  }
</style>
