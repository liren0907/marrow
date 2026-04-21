<script lang="ts">
    import type { Snippet } from "svelte";
    import Icon, { type IconName } from "./Icon.svelte";

    let {
        title = undefined,
        icon = undefined,
        variant = "base-100",
        shadow = "sm",
        bordered = true,
        compact = false,
        class: className = "",
        children,
    }: {
        title?: string;
        icon?: IconName;
        variant?: "base-100" | "base-200";
        shadow?: "none" | "sm" | "md" | "lg" | "xl";
        bordered?: boolean;
        compact?: boolean;
        class?: string;
        children: Snippet;
    } = $props();

    let shadowClass = $derived(shadow === "none" ? "" : `shadow-${shadow}`);
    let borderClass = $derived(bordered ? "border border-base-200" : "");
    let compactClass = $derived(compact ? "card-compact" : "");
</script>

<div class="card bg-{variant} {shadowClass} {borderClass} {compactClass} {className}">
    {#if title}
        <div class="card-body">
            <h2 class="card-title text-sm font-medium flex items-center gap-2">
                {#if icon}
                    <Icon name={icon} size={16} />
                {/if}
                {title}
            </h2>
            {@render children()}
        </div>
    {:else}
        {@render children()}
    {/if}
</div>
