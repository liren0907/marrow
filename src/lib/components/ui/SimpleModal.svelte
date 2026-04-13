<script lang="ts">
    import type { Snippet } from "svelte";
    import { scale } from "svelte/transition";
    import IconButton from "./IconButton.svelte";

    let {
        isOpen = false,
        title,
        maxWidth = "max-w-md",
        onclose,
        children,
        actions,
    }: {
        isOpen?: boolean;
        title: string;
        maxWidth?: string;
        onclose?: () => void;
        children?: Snippet;
        actions?: Snippet;
    } = $props();

    // Close on escape
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && isOpen) {
            close();
        }
    }

    function close() {
        onclose?.();
    }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
    <div
        class="modal modal-open z-[60]"
        role="dialog"
        aria-labelledby="modal-title"
    >
        <!-- Backdrop -->
        <button
            type="button"
            class="modal-backdrop cursor-default"
            onclick={close}
            onkeydown={(e) => e.key === "Enter" && close()}
            aria-label="Close modal"
        ></button>

        <!-- Modal Box -->
        <div
            class="modal-box {maxWidth} p-0 bg-base-100 border border-base-300 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
            transition:scale={{ start: 0.95, duration: 200 }}
        >
            <!-- Header -->
            <div
                class="px-6 py-4 flex items-center justify-between border-b border-base-200 bg-base-100 sticky top-0 z-10"
            >
                <h3 id="modal-title" class="font-bold text-lg">{title}</h3>
                <IconButton icon="close" size="sm" onclick={close} />
            </div>

            <!-- Content (Scrollable) -->
            <div class="p-6 overflow-y-auto flex-1">
                {@render children?.()}
            </div>

            <!-- Actions (Footer) -->
            {#if actions}
                <div
                    class="px-6 py-4 bg-base-200/50 border-t border-base-200 flex justify-end items-center gap-3"
                >
                    {@render actions()}
                </div>
            {/if}
        </div>
    </div>
{/if}
