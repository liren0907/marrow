<script lang="ts">
  import {
    editorSettings,
    setAutosaveDebounce,
    setPdfZoom,
    setWikiSuggestionCount,
    setTransclusionSuggestionCount,
    AUTOSAVE_DEBOUNCE_MIN,
    AUTOSAVE_DEBOUNCE_MAX,
    PDF_ZOOM_MIN,
    PDF_ZOOM_MAX,
    SUGGESTION_COUNT_MIN,
    SUGGESTION_COUNT_MAX,
  } from "../editorSettings.svelte";

  function onAutosaveInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setAutosaveDebounce(v);
  }
  function onPdfZoomInput(e: Event) {
    const v = parseFloat((e.target as HTMLInputElement).value);
    if (Number.isFinite(v)) setPdfZoom(v);
  }
  function onWikiCountInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setWikiSuggestionCount(v);
  }
  function onTransCountInput(e: Event) {
    const v = parseInt((e.target as HTMLInputElement).value, 10);
    if (Number.isFinite(v)) setTransclusionSuggestionCount(v);
  }
</script>

<div class="section">
  <h3 class="section-title">Autosave delay</h3>
  <p class="section-desc">
    How long the editor waits after you stop typing before writing to disk.
    Lower = faster persistence but more disk churn. Takes effect on the next
    edit.
  </p>
  <div class="row">
    <input
      type="range"
      min={AUTOSAVE_DEBOUNCE_MIN}
      max={AUTOSAVE_DEBOUNCE_MAX}
      step="100"
      value={editorSettings.autosaveDebounceMs}
      oninput={onAutosaveInput}
    />
    <span class="value">{editorSettings.autosaveDebounceMs} ms</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">PDF default zoom</h3>
  <p class="section-desc">
    Initial render scale for PDF pages. Larger values look sharper on hi-DPI
    screens but use more memory. Applies to PDFs opened after the change.
  </p>
  <div class="row">
    <input
      type="range"
      min={PDF_ZOOM_MIN}
      max={PDF_ZOOM_MAX}
      step="0.1"
      value={editorSettings.pdfZoom}
      oninput={onPdfZoomInput}
    />
    <span class="value">{editorSettings.pdfZoom.toFixed(1)}×</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Wiki-link suggestion count</h3>
  <p class="section-desc">
    How many filenames to show in the <code>[[…]]</code> autocomplete popup.
    Higher = more options, more scrolling.
  </p>
  <div class="row">
    <input
      type="range"
      min={SUGGESTION_COUNT_MIN}
      max={SUGGESTION_COUNT_MAX}
      step="1"
      value={editorSettings.wikiSuggestionCount}
      oninput={onWikiCountInput}
    />
    <span class="value">{editorSettings.wikiSuggestionCount}</span>
  </div>
</div>

<div class="section">
  <h3 class="section-title">Transclusion suggestion count</h3>
  <p class="section-desc">
    Same as above but for the <code>![[…]]</code> embed popup, which can
    also list images, audio, and video.
  </p>
  <div class="row">
    <input
      type="range"
      min={SUGGESTION_COUNT_MIN}
      max={SUGGESTION_COUNT_MAX}
      step="1"
      value={editorSettings.transclusionSuggestionCount}
      oninput={onTransCountInput}
    />
    <span class="value">{editorSettings.transclusionSuggestionCount}</span>
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
  code {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--color-base-300);
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--color-base-content);
  }
</style>
