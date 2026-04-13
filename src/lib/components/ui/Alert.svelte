<script lang="ts">
    import type { Snippet } from "svelte";
    import { fly } from "svelte/transition";

    let {
        variant = "info",
        title,
        dismissible = false,
        icon,
        onclose,
        children,
    }: {
        variant?: "info" | "success" | "warning" | "error";
        title?: string;
        dismissible?: boolean;
        icon?: string;
        onclose?: () => void;
        children?: Snippet;
    } = $props();

    // Mapping variants to icons if not provided
    const defaultIcons = {
        info: "info",
        success: "check_circle",
        warning: "warning",
        error: "error",
    };

    let resolvedIcon = $derived(icon || defaultIcons[variant]);

    let alertClass = $derived(
        {
            info: "alert-info",
            success: "alert-success",
            warning: "alert-warning",
            error: "alert-error",
        }[variant],
    );
</script>

<div
    role="alert"
    class="alert {alertClass} shadow-sm"
    transition:fly={{ y: -10, duration: 200 }}
>
    <span class="material-symbols-rounded">{resolvedIcon}</span>

    <div class="flex-1">
        {#if title}
            <h3 class="font-bold">{title}</h3>
        {/if}
        <div class="text-sm">
            {@render children?.()}
        </div>
    </div>

    {#if dismissible}
        <button
            class="btn btn-sm btn-circle btn-ghost"
            onclick={() => onclose?.()}
        >
            <span class="material-symbols-rounded">close</span>
        </button>
    {/if}
</div>
