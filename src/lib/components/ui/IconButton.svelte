<script lang="ts">
    let {
        icon,
        label = undefined,
        tooltip = undefined,
        active = false,
        loading = false,
        size = "sm",
        variant = "ghost",
        bordered = false,
        responsiveLabel = false,
        disabled = false,
        class: className = "",
        onclick,
    }: {
        icon: string;
        label?: string | undefined;
        tooltip?: string | undefined;
        active?: boolean;
        loading?: boolean;
        size?: "sm" | "md" | "lg";
        variant?: "ghost" | "soft";
        bordered?: boolean;
        responsiveLabel?: boolean;
        disabled?: boolean;
        class?: string;
        onclick?: (event: MouseEvent) => void;
    } = $props();

    function handleClick(event: MouseEvent) {
        if (!disabled && !loading) {
            onclick?.(event);
        }
    }

    let sizeClass = $derived(
        size === "sm"
            ? "btn-sm text-lg"
            : size === "md"
              ? "btn-md text-xl"
              : "btn-lg text-2xl",
    );

    let borderStyle = $derived(bordered ? "border border-base-300" : "");

    let variantClass = $derived(
        variant === "ghost"
            ? `btn-ghost text-base-content/70 hover:bg-base-200 ${borderStyle}`
            : `bg-base-200 text-base-content hover:bg-base-300 ${bordered ? borderStyle : "border-none"}`,
    );

    let activeClass = $derived(
        active
            ? "bg-primary text-primary-content hover:bg-primary hover:text-primary-content shadow-inner"
            : "",
    );

    // Use btn-square only when no label
    let shapeClass = $derived(label ? "gap-2 px-3" : "btn-square");
</script>

<div class="tooltip tooltip-bottom" data-tip={tooltip}>
    <button
        type="button"
        class="btn {sizeClass} {variantClass} {activeClass} {shapeClass} {className}"
        {disabled}
        class:loading
        onclick={handleClick}
    >
        {#if !loading}
            <span class="material-symbols-rounded">{icon}</span>
            {#if label}
                <span class="text-sm font-medium {responsiveLabel ? 'hidden xl:inline' : ''}">{label}</span>
            {/if}
        {/if}
    </button>
</div>
