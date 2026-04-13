<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { PDFDocumentProxy } from "pdfjs-dist";
  import PdfPage from "./PdfPage.svelte";

  let { pdfDoc }: { pdfDoc: PDFDocumentProxy } = $props();

  let scrollEl: HTMLDivElement;
  let visible = $state<Set<number>>(new Set());
  let observers: IntersectionObserver | null = null;
  const placeholderHeight = 842; // ~A4 at scale 1.0; will be replaced when page renders

  let pageNumbers = $derived(
    Array.from({ length: pdfDoc.numPages }, (_, i) => i + 1),
  );
  let currentPage = $state(1);

  onMount(() => {
    observers = new IntersectionObserver(
      (entries) => {
        const next = new Set(visible);
        let lastIntersecting = currentPage;
        for (const entry of entries) {
          const num = Number((entry.target as HTMLElement).dataset.page);
          if (entry.isIntersecting) {
            next.add(num);
            // Lookahead ± 1
            if (num > 1) next.add(num - 1);
            if (num < pdfDoc.numPages) next.add(num + 1);
            lastIntersecting = num;
          } else {
            next.delete(num);
          }
        }
        visible = next;
        currentPage = lastIntersecting;
      },
      { root: scrollEl, rootMargin: "200px 0px" },
    );
    queueMicrotask(() => {
      scrollEl
        .querySelectorAll<HTMLElement>("[data-page]")
        .forEach((el) => observers?.observe(el));
    });
  });

  onDestroy(() => {
    observers?.disconnect();
    observers = null;
  });
</script>

<div class="pdf-viewer flex flex-col h-full">
  <div bind:this={scrollEl} class="flex-1 overflow-y-auto bg-base-200 py-6">
    <div class="flex flex-col items-center gap-4">
      {#each pageNumbers as num (num)}
        <div data-page={num} style:min-height="{placeholderHeight}px">
          {#if visible.has(num)}
            <PdfPage {pdfDoc} pageNumber={num} />
          {/if}
        </div>
      {/each}
    </div>
  </div>
  <div
    class="px-3 py-1 border-t border-base-200 text-xs text-base-content/60 flex justify-center bg-base-100"
  >
    Page {currentPage} / {pdfDoc.numPages}
  </div>
</div>
