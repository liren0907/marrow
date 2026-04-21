<script lang="ts">
    import type { Snippet } from "svelte";
    import { fly } from "svelte/transition";
    import Icon, { type IconName } from "./Icon.svelte";

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
        icon?: IconName;
        onclose?: () => void;
        children?: Snippet;
    } = $props();

    const defaultIcons: Record<"info" | "success" | "warning" | "error", IconName> = {
        info: "info",
        success: "circle-check",
        warning: "triangle-alert",
        error: "circle-alert",
    };

    let resolvedIcon: IconName = $derived(icon || defaultIcons[variant]);

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
    <Icon name={resolvedIcon} size={20} />

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
            <Icon name="x" size={18} />
        </button>
    {/if}
</div>
