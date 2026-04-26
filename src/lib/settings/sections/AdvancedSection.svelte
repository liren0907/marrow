<script lang="ts">
  import {
    advancedSettings,
    setImagePasteMaxBytes,
    setEmbedMaxDepth,
    setEmbedRenderBytes,
    setSearchResultLimit,
    setDiffLineCap,
    resetAdvancedSettings,
    IMAGE_PASTE_MAX_MIN,
    IMAGE_PASTE_MAX_MAX,
    EMBED_DEPTH_MIN,
    EMBED_DEPTH_MAX,
    EMBED_RENDER_BYTES_MIN,
    EMBED_RENDER_BYTES_MAX,
    SEARCH_LIMIT_MIN,
    SEARCH_LIMIT_MAX,
    DIFF_LINE_CAP_MIN,
    DIFF_LINE_CAP_MAX,
  } from "../advancedSettings.svelte";

  function onImageMax(e: Event) {
    const mb = parseFloat((e.target as HTMLInputElement).value);
    if (Number.isFinite(mb)) setImagePasteMaxBytes(mb * 1024 * 1024);
  }
  function onDepth(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setEmbedMaxDepth(v);
  }
  function onRenderBytes(e: Event) {
    const kb = parseFloat((e.target as HTMLInputElement).value);
    if (Number.isFinite(kb)) setEmbedRenderBytes(kb * 1024);
  }
  function onSearchLimit(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setSearchResultLimit(v);
  }
  function onDiffCap(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setDiffLineCap(v);
  }

  // Display helpers — store keeps bytes, sliders work in MB / KB.
  const imageMaxMb = $derived(advancedSettings.imagePasteMaxBytes / 1024 / 1024);
  const renderBytesKb = $derived(advancedSettings.embedRenderBytes / 1024);
</script>

<div class="warn">
  <strong>Heads up:</strong> these limits exist to keep the editor responsive.
  Raising them past the defaults is fine on fast hardware but can cause lag,
  high memory use, or runaway recursion on large workspaces.
</div>

<div class="section">
  <h3 class="section-title">Image paste max size</h3>
  <p class="section-desc">
    Pasted images larger than this are rejected with a toast. Helps avoid
    accidentally pasting a 200MB screenshot.
  </p>
  <div class="row">
    <input
      type="range"
      min={IMAGE_PASTE_MAX_MIN / 1024 / 1024}
      max={IMAGE_PASTE_MAX_MAX / 1024 / 1024}
      step="1"
      value={imageMaxMb}
      oninput={onImageMax}
    />
    <span class="value">{imageMaxMb.toFixed(0)} MB</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Embed nesting depth</h3>
  <p class="section-desc">
    Maximum levels of <code>![[…]]</code> embeds allowed. Beyond this depth a
    placeholder is shown instead — prevents infinite recursion when files
    embed each other.
  </p>
  <div class="row">
    <input
      type="range"
      min={EMBED_DEPTH_MIN}
      max={EMBED_DEPTH_MAX}
      step="1"
      value={advancedSettings.embedMaxDepth}
      oninput={onDepth}
    />
    <span class="value">{advancedSettings.embedMaxDepth} level{advancedSettings.embedMaxDepth === 1 ? "" : "s"}</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Embed render size cap</h3>
  <p class="section-desc">
    A single embedded file larger than this isn't rendered (a "too large"
    notice appears instead). Affects the preview only — the source file is
    untouched.
  </p>
  <div class="row">
    <input
      type="range"
      min={EMBED_RENDER_BYTES_MIN / 1024}
      max={EMBED_RENDER_BYTES_MAX / 1024}
      step="10"
      value={renderBytesKb}
      oninput={onRenderBytes}
    />
    <span class="value">
      {renderBytesKb >= 1024
        ? `${(renderBytesKb / 1024).toFixed(1)} MB`
        : `${renderBytesKb.toFixed(0)} KB`}
    </span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Search result limit</h3>
  <p class="section-desc">
    Max hits returned per full-text search query. Higher = more complete
    results but slower rendering for huge workspaces.
  </p>
  <div class="row">
    <input
      type="range"
      min={SEARCH_LIMIT_MIN}
      max={SEARCH_LIMIT_MAX}
      step="50"
      value={advancedSettings.searchResultLimit}
      oninput={onSearchLimit}
    />
    <span class="value">{advancedSettings.searchResultLimit}</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Diff display line cap</h3>
  <p class="section-desc">
    Per-side limit before the conflict modal's diff is hidden behind a
    "too large to display" notice. Larger diffs are still resolvable —
    just not visually shown.
  </p>
  <div class="row">
    <input
      type="range"
      min={DIFF_LINE_CAP_MIN}
      max={DIFF_LINE_CAP_MAX}
      step="500"
      value={advancedSettings.diffLineCap}
      oninput={onDiffCap}
    />
    <span class="value">{advancedSettings.diffLineCap.toLocaleString()} lines</span>
  </div>
</div>

<div class="reset-row">
  <button class="reset-btn" onclick={resetAdvancedSettings}>
    Reset Advanced to defaults
  </button>
</div>

<style>
  .warn {
    background: color-mix(in oklch, oklch(0.7 0.15 65) 12%, transparent);
    border: 1px solid color-mix(in oklch, oklch(0.7 0.15 65) 35%, transparent);
    border-radius: var(--mw-radius-sm);
    padding: 10px 12px;
    margin-bottom: 20px;
    font-size: 12px;
    color: var(--color-base-content);
    line-height: 1.5;
  }
  .warn strong {
    color: oklch(0.6 0.18 65);
  }
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
    margin: 0 0 12px;
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
    min-width: 80px;
    text-align: right;
  }
  code {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--color-base-300);
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--color-base-content);
  }
  .reset-row {
    margin-top: 28px;
    padding-top: 16px;
    border-top: 1px solid var(--mw-rule);
  }
  .reset-btn {
    background: var(--color-base-200);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 6px 12px;
    font-size: 12px;
    color: var(--mw-ink-2);
    cursor: pointer;
  }
  .reset-btn:hover {
    background: var(--color-base-300);
    color: var(--color-base-content);
  }
</style>
