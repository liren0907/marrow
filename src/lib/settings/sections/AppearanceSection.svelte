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

  const accentEntries: { key: AccentKey; label: string }[] = (
    Object.keys(ACCENTS) as AccentKey[]
  ).map((k) => ({ key: k, label: ACCENTS[k].label }));

  // Theme is stored in localStorage and reflected on <html data-theme>.
  // Mirrors the logic in TweaksPanel — kept duplicated rather than
  // extracted because the two surfaces stay decoupled.
  let theme = $state<"light" | "dark">("light");

  $effect(() => {
    const t = document.documentElement.getAttribute("data-theme") ?? "";
    theme = t === "marrow-pro-dark" || t === "dark" ? "dark" : "light";
  });

  function setTheme(next: "light" | "dark") {
    theme = next;
    const name = next === "dark" ? "marrow-pro-dark" : "marrow-pro-light";
    document.documentElement.setAttribute("data-theme", name);
    localStorage.setItem("theme", name);
  }
</script>

<div class="section">
  <h3 class="section-title">Theme</h3>
  <p class="section-desc">Choose the overall light or dark color scheme.</p>
  <div class="seg">
    <button class:on={theme === "light"} onclick={() => setTheme("light")}>
      Light
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
        style:background={accentColor(entry.key, theme)}
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
</style>
