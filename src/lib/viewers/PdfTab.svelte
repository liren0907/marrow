<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { getDocument, type PDFDocumentProxy } from "pdfjs-dist";
  import type { Tab } from "$lib/workspace/types";
  import { readBinaryFile } from "$lib/workspace/tauri";
  import PdfViewer from "./PdfViewer/PdfViewer.svelte";
  import { ensurePdfWorker } from "./PdfViewer/usePdfWorker";

  let { tab }: { tab: Tab } = $props();

  let pdfDoc: PDFDocumentProxy | null = null;
  // $state.raw avoids deep-proxying the PDFDocumentProxy native handles.
  let pdfDocReactive = $state.raw<PDFDocumentProxy | null>(null);
  let loadError = $state<string | null>(null);
  let cancelled = false;

  onMount(() => {
    ensurePdfWorker();
    (async () => {
      try {
        const bytes = await readBinaryFile(tab.path);
        if (cancelled) return;
        const doc = await getDocument({ data: bytes }).promise;
        if (cancelled) {
          await doc.destroy();
          return;
        }
        pdfDoc = doc;
        pdfDocReactive = doc;
      } catch (e) {
        loadError = e instanceof Error ? e.message : String(e);
        console.error("[pdf] load failed", e);
      }
    })();
  });

  onDestroy(() => {
    cancelled = true;
    if (pdfDoc) {
      try {
        void pdfDoc.destroy();
      } catch {
        // ignore
      }
      pdfDoc = null;
    }
  });
</script>

<div class="w-full h-full">
  {#if loadError}
    <div class="p-6 text-error text-sm">Failed to load PDF: {loadError}</div>
  {:else if pdfDocReactive}
    <PdfViewer pdfDoc={pdfDocReactive} />
  {:else}
    <div class="p-6 text-base-content/40 text-sm">Loading PDF…</div>
  {/if}
</div>
