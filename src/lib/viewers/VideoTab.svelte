<script lang="ts">
  import type { Tab } from "$lib/workspace/types";
  import { safeConvertFileSrc } from "$lib/utils/tauriUtils";

  let { tab }: { tab: Tab } = $props();
  const url = $derived(safeConvertFileSrc(tab.path));
</script>

<div class="w-full h-full flex items-center justify-center bg-black">
  <!-- svelte-ignore a11y_media_has_caption -->
  <video src={url} controls class="max-w-full max-h-full"></video>
</div>
