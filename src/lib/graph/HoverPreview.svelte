<script lang="ts">
  // Floating preview card for graph node hover. State + scheduling lives in
  // hoverPreview.svelte.ts — this file is purely presentational.
  import { hoverState } from "./hoverPreviewState.svelte";
</script>

{#if hoverState.path}
  <div
    class="fixed pointer-events-none z-30 max-w-[320px] bg-base-100/95 border border-base-300 rounded-md shadow-lg p-3 text-[11px] text-base-content/80 leading-relaxed backdrop-blur"
    style:left="{hoverState.pos.x}px"
    style:top="{hoverState.pos.y}px"
  >
    <div class="font-semibold text-base-content/60 text-[10px] mb-1 truncate">
      {hoverState.path.split(/[/\\]/).pop()}
    </div>
    <div class="line-clamp-6 whitespace-pre-wrap break-words">
      {hoverState.content || "(empty)"}
    </div>
  </div>
{/if}
