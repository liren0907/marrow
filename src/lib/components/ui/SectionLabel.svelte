<script lang="ts">
    import type { Snippet } from "svelte";

    /**
     * SectionLabel Component - /gallery Style
     * Uppercase tracking-wider label for section headers.
     */

    let { children }: { children?: Snippet } = $props();
</script>

<label class="text-xs font-bold text-base-content/40 uppercase tracking-wider">
    {@render children?.()}
</label>
