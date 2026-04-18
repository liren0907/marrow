<script lang="ts">
    import type { Snippet } from "svelte";

    /**
     * Button Component - /gallery Style
     * Uses the Flat Gray aesthetic from /gallery ExportModal.
     */

    let {
        variant = "default",
        size = "sm",
        disabled = false,
        type = "button",
        minWidth = null,
        class: className = "",
        onclick,
        children,
    }: {
        variant?: "default" | "ghost";
        size?: "xs" | "sm" | "md";
        disabled?: boolean;
        type?: "button" | "submit" | "reset";
        minWidth?: string | null;
        class?: string;
        onclick?: (event: MouseEvent) => void;
        children?: Snippet;
    } = $props();

    function handleClick(event: MouseEvent) {
        if (!disabled) {
            onclick?.(event);
        }
    }

    let baseClass = $derived("btn border-none font-normal transition-all");

    let sizeClass = $derived(
        size === "xs" ? "btn-xs" : size === "sm" ? "btn-sm" : "",
    );

    let variantClass = $derived(
        variant === "ghost"
            ? "btn-ghost bg-base-200/60 hover:bg-base-300/70 text-base-content/70"
            : "bg-base-200 hover:bg-base-300 text-base-content",
    );

    let style = $derived(minWidth ? `min-width: ${minWidth};` : "");
</script>

<button
    {type}
    class="{baseClass} {sizeClass} {variantClass} {className}"
    {style}
    {disabled}
    onclick={handleClick}
>
    {@render children?.()}
</button>
