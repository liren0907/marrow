<script lang="ts">
    import type { Snippet } from "svelte";

    let {
        variant = undefined,
        size = undefined,
        class: className = "",
        children,
    }: {
        variant?: "neutral" | "primary" | "secondary" | "accent" | "success" | "warning" | "info" | "ghost" | "outline";
        size?: "xs" | "sm" | "md" | "lg";
        class?: string;
        children: Snippet;
    } = $props();

    let variantClass = $derived(variant ? `badge-${variant}` : "");
    let sizeClass = $derived(size ? `badge-${size}` : "");
</script>

<span class="badge {variantClass} {sizeClass} {className}">
    {@render children()}
</span>
