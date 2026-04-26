<script lang="ts">
  /*
   * Generic popover — pure JS state, no CSS focus games.
   *
   * Why this exists: DaisyUI's `.dropdown` uses `:focus-within` which
   * silently breaks when paired with libraries that call
   * `event.preventDefault()` on canvas mousedown (cytoscape, PixiJS,
   * Konva, etc). preventDefault blocks the browser's default focus
   * transfer, so clicking on the canvas to dismiss a dropdown does
   * nothing — the panel stays open because focus is still inside it.
   *
   * This component sidesteps the whole class of bugs by tracking
   * open/close in JS state. Click-outside is an explicit window
   * mousedown handler; Esc is an explicit keydown handler.
   *
   * Companion of TweaksPanel / ContextMenu / SimpleModal — all of
   * which already follow this self-rolled pattern. GraphToolbar was
   * the lone outlier still riding DaisyUI dropdowns; this fixes it.
   */
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    align = "start",
    triggerClass = "",
    panelClass = "",
    triggerAriaLabel,
    trigger,
    children,
  }: {
    /** Two-way bindable open state. */
    open?: boolean;
    /** Panel alignment relative to trigger. `end` anchors to right edge. */
    align?: "start" | "end";
    /** Extra classes for the trigger <button>. Apply your existing styles here. */
    triggerClass?: string;
    /** Extra classes for the panel <div>. */
    panelClass?: string;
    /** Optional aria-label for the trigger (icon-only triggers should set this). */
    triggerAriaLabel?: string;
    /** Snippet rendered inside the trigger button. */
    trigger: Snippet;
    /** Snippet rendered inside the panel (only when open). */
    children: Snippet;
  } = $props();

  let triggerEl: HTMLButtonElement | undefined = $state();
  let panelEl: HTMLDivElement | undefined = $state();

  function toggle() {
    open = !open;
  }

  function onWindowMouseDown(e: MouseEvent) {
    if (!open) return;
    const t = e.target as Node | null;
    if (!t) return;
    // Don't close if click is on the trigger (toggle handler will do it),
    // or anywhere inside the panel (selecting an option shouldn't dismiss).
    if (triggerEl?.contains(t)) return;
    if (panelEl?.contains(t)) return;
    open = false;
  }

  function onWindowKeyDown(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === "Escape") {
      open = false;
      // Stop other Esc handlers (e.g. modal close) firing in the same tick.
      e.stopPropagation();
    }
  }
</script>

<svelte:window onmousedown={onWindowMouseDown} onkeydown={onWindowKeyDown} />

<div class="popover-wrap">
  <button
    bind:this={triggerEl}
    type="button"
    class="popover-trigger {triggerClass}"
    aria-expanded={open}
    aria-haspopup="true"
    aria-label={triggerAriaLabel}
    onclick={toggle}
  >
    {@render trigger()}
  </button>
  {#if open}
    <div
      bind:this={panelEl}
      class="popover-panel {panelClass}"
      class:end={align === "end"}
      role="dialog"
    >
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .popover-wrap {
    position: relative;
    display: inline-block;
  }
  /* Reset native button defaults so consumer's triggerClass styles win.
     Visual styling lives in the consumer-provided class. */
  .popover-trigger {
    appearance: none;
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
    text-align: left;
    line-height: 1;
  }
  .popover-trigger:focus {
    outline: none;
  }
  .popover-trigger:focus-visible {
    outline: 2px solid var(--mw-accent);
    outline-offset: 1px;
    border-radius: var(--mw-radius-xs);
  }
  .popover-panel {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    z-index: 999;
    /* Visual styling (background, border, shadow, padding) intentionally
       left to consumer via panelClass — keeps this component
       presentation-agnostic. */
  }
  .popover-panel.end {
    left: auto;
    right: 0;
  }
</style>
