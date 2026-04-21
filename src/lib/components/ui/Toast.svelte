<script lang="ts">
    import { toasts, dismissToast } from "$lib/stores/toastStore.svelte";
    import { fly } from "svelte/transition";
    import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";

    const variantStyles = {
        success: "alert-success",
        error: "alert-error",
        info: "alert-info",
        warning: "alert-warning",
    };

    const variantIcons: Record<"success" | "error" | "info" | "warning", IconName> = {
        success: "circle-check",
        error: "circle-alert",
        info: "info",
        warning: "triangle-alert",
    };
</script>

{#if toasts.length > 0}
    <div class="toast toast-top toast-end z-50">
        {#each toasts as toast (toast.id)}
            <div
                class="alert {variantStyles[toast.variant]} shadow-lg"
                transition:fly={{ x: 100, duration: 300 }}
            >
                <Icon name={variantIcons[toast.variant]} size={18} />
                <span>{toast.message}</span>
                <button
                    class="btn btn-ghost btn-xs"
                    onclick={() => dismissToast(toast.id)}
                >
                    <Icon name="x" size={14} />
                </button>
            </div>
        {/each}
    </div>
{/if}
