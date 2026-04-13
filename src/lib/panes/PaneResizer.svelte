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
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pane-resizer"
  class:dragging
  onmousedown={onMouseDown}
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
  .pane-resizer.dragging {
    background-color: oklch(var(--p) / 0.4);
  }
</style>
