<script lang="ts">
  import type { Pane as PaneType } from "$lib/workspace/types";
  import MarkdownTab from "$lib/viewers/MarkdownTab.svelte";
  import ImageTab from "$lib/viewers/ImageTab.svelte";
  import VideoTab from "$lib/viewers/VideoTab.svelte";
  import AudioTab from "$lib/viewers/AudioTab.svelte";
  import PdfTab from "$lib/viewers/PdfTab.svelte";
  import TextTab from "$lib/viewers/TextTab.svelte";
  import GraphTab from "$lib/graph/GraphTab.svelte";
  import UnsupportedTab from "$lib/viewers/UnsupportedTab.svelte";

  let { pane }: { pane: PaneType } = $props();
</script>

{#each pane.tabs as tab (tab.id)}
  <div class="absolute inset-0" class:hidden={tab.id !== pane.activeTabId}>
    {#if tab.kind === "markdown"}
      <MarkdownTab {tab} />
    {:else if tab.kind === "image"}
      <ImageTab {tab} />
    {:else if tab.kind === "video"}
      <VideoTab {tab} />
    {:else if tab.kind === "audio"}
      <AudioTab {tab} />
    {:else if tab.kind === "pdf"}
      <PdfTab {tab} />
    {:else if tab.kind === "text"}
      <TextTab {tab} />
    {:else if tab.kind === "graph"}
      <GraphTab {tab} />
    {:else}
      <UnsupportedTab {tab} />
    {/if}
  </div>
{/each}
