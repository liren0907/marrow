<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
  import { editorSettings } from "$lib/settings/editorSettings.svelte";

  let {
    pdfDoc,
    pageNumber,
    scale,
  }: {
    pdfDoc: PDFDocumentProxy;
    pageNumber: number;
    /** Override the user-configured zoom for this page (rare). */
    scale?: number;
  } = $props();

  // Use the user-configured zoom unless an explicit override is passed.
  // The renderer only reads this once in onMount, so subsequent setting
  // changes apply on the next PDF tab open rather than mid-page re-render.
  const effectiveScale = $derived(scale ?? editorSettings.pdfZoom);

  let canvas: HTMLCanvasElement;
  let renderTask: RenderTask | null = null;
  let cancelled = false;
  let dimensions = $state<{ w: number; h: number } | null>(null);

  onMount(() => {
    (async () => {
      try {
        const page = await pdfDoc.getPage(pageNumber);
        if (cancelled) {
          page.cleanup();
          return;
        }
        const viewport = page.getViewport({ scale: effectiveScale });
        dimensions = { w: viewport.width, h: viewport.height };
        // Wait a tick so canvas size is committed to the DOM.
        await Promise.resolve();
        if (cancelled) {
          page.cleanup();
          return;
        }
        const ctx = canvas.getContext("2d");
        if (!ctx) return;
        canvas.width = viewport.width;
        canvas.height = viewport.height;
        renderTask = page.render({ canvasContext: ctx, viewport, canvas });
        try {
          await renderTask.promise;
        } catch (e: unknown) {
          // RenderingCancelledException is expected on tab switch.
          if ((e as { name?: string }).name !== "RenderingCancelledException") {
            console.warn("[pdf] render failed", e);
          }
        }
        page.cleanup();
      } catch (e) {
        console.warn("[pdf] page load failed", e);
      }
    })();
  });

  onDestroy(() => {
    cancelled = true;
    try {
      renderTask?.cancel();
    } catch {
      // ignore
    }
  });
</script>

<div
  class="pdf-page-wrap"
  style:width={dimensions ? `${dimensions.w}px` : "595px"}
  style:height={dimensions ? `${dimensions.h}px` : "842px"}
>
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .pdf-page-wrap {
    background: white;
    box-shadow: 0 2px 8px oklch(0 0 0 / 0.15);
    margin: 0 auto;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
