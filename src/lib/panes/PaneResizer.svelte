<script lang="ts">
  import { workspace } from "$lib/workspace/workspace.svelte";

  let dragging = $state(false);

  function onMouseDown(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
    const startX = e.clientX;
    const startRatio = workspace.splitRatio;
    const containerWidth =
      (e.currentTarget as HTMLElement).parentElement?.clientWidth ?? 1;

    function onMove(ev: MouseEvent) {
      const dx = ev.clientX - startX;
      workspace.setSplitRatio(startRatio + dx / containerWidth);
    }
    function onUp() {
      dragging = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  function onKeyDown(e: KeyboardEvent) {
    // ~2% step on each tap; Shift = ~5%.
    const step = e.shiftKey ? 0.05 : 0.02;
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      workspace.setSplitRatio(workspace.splitRatio - step);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      workspace.setSplitRatio(workspace.splitRatio + step);
    } else if (e.key === "Home") {
      e.preventDefault();
      workspace.setSplitRatio(0.5);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="pane-resizer"
  class:dragging
  onmousedown={onMouseDown}
  onkeydown={onKeyDown}
  role="separator"
  aria-orientation="vertical"
  aria-label="Resize pane split"
  aria-valuenow={Math.round(workspace.splitRatio * 100)}
  aria-valuemin="15"
  aria-valuemax="85"
  tabindex="0"
></div>

<style>
  .pane-resizer {
    width: 4px;
    cursor: col-resize;
    background-color: oklch(var(--b2));
    transition: background-color 0.15s;
    flex-shrink: 0;
  }
  .pane-resizer:hover,
  .pane-resizer.dragging,
  .pane-resizer:focus-visible {
    background-color: oklch(var(--p) / 0.4);
    outline: none;
  }
</style>
