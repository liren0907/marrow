<script lang="ts">
  import type { Tab } from "$lib/workspace/types";
  import { safeConvertFileSrc } from "$lib/utils/tauriUtils";

  let { tab }: { tab: Tab } = $props();
  const url = $derived(safeConvertFileSrc(tab.path));
</script>

<div
  class="w-full h-full flex items-center justify-center bg-base-200 overflow-auto p-4"
>
  <img
    src={url}
    alt={tab.title}
    class="max-w-full max-h-full object-contain"
  />
</div>
