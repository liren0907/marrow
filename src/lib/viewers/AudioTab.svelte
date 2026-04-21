<script lang="ts">
  import type { Tab } from "$lib/workspace/types";
  import { safeConvertFileSrc } from "$lib/utils/tauriUtils";
  import Icon from "$lib/components/ui/Icon.svelte";

  let { tab }: { tab: Tab } = $props();
  const url = $derived(safeConvertFileSrc(tab.path));
</script>

<div class="w-full h-full flex items-center justify-center bg-base-200">
  <div class="flex flex-col items-center gap-4">
    <Icon name="music" size={64} strokeWidth={1.25} class="text-base-content/40" />
    <p class="text-sm font-mono">{tab.title}</p>
    <audio src={url} controls></audio>
  </div>
</div>
