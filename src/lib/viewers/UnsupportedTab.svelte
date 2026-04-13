<script lang="ts">
  import type { Tab } from "$lib/workspace/types";
  import { open } from "@tauri-apps/plugin-shell";
  import { Button } from "$lib/components/ui";
  import { showError } from "$lib/stores/toastStore.svelte";

  let { tab }: { tab: Tab } = $props();

  const kindLabel = (() => {
    switch (tab.kind) {
      case "pdf":
        return "PDF";
      case "text":
        return "Text / Code";
      default:
        return "Unsupported file";
    }
  })();

  async function openWithSystem() {
    try {
      await open(tab.path);
    } catch (e) {
      showError(
        `Failed to open: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }
</script>

<div class="w-full h-full flex items-center justify-center bg-base-100">
  <div class="flex flex-col items-center gap-4 text-center">
    <span class="material-symbols-rounded text-[64px] text-base-content/30"
      >draft</span
    >
    <div>
      <p class="text-sm font-mono">{tab.title}</p>
      <p class="text-xs text-base-content/40 mt-1">{kindLabel}</p>
    </div>
    <Button onclick={openWithSystem}>
      <span class="material-symbols-rounded text-[16px] mr-1">open_in_new</span>
      Open with system
    </Button>
  </div>
</div>
