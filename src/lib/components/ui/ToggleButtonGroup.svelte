<script lang="ts">
    import Icon, { type IconName } from "./Icon.svelte";

    let {
        options = [],
        value = $bindable(),
        size = "sm",
        responsiveLabels = false,
        tooltipPosition = "top",
        onchange,
    }: {
        options?: Array<{
            value: any;
            label?: string;
            icon?: IconName;
            tooltip?: string;
        }>;
        value?: any;
        size?: "sm" | "md";
        responsiveLabels?: boolean;
        tooltipPosition?: "top" | "bottom" | "left" | "right";
        onchange?: (value: any) => void;
    } = $props();

    const tooltipClass = $derived(
        tooltipPosition === "top" ? "tooltip" : `tooltip tooltip-${tooltipPosition}`,
    );

    function select(optionValue: any) {
        if (value !== optionValue) {
            value = optionValue;
            onchange?.(value);
        }
    }

    let sizeClass = $derived(size === "sm" ? "btn-sm" : "btn-md");
</script>

<div class="join border border-base-300 bg-base-100 rounded-[var(--mw-radius-sm)] p-1 space-x-1">
    {#each options as option}
        <div class={tooltipClass} data-tip={option.tooltip}>
            <button
                class="btn {sizeClass} join-item border-none hover:bg-base-200 transition-all duration-200"
                class:btn-square={option.icon && !option.label}
                class:gap-2={option.label}
                class:px-4={option.label}
                class:bg-base-200={value === option.value}
                class:shadow-inner={value === option.value}
                class:text-primary={value === option.value && !option.label}
                class:text-base-content={value === option.value && !!option.label}
                class:text-base-content-secondary={value !== option.value}
                onclick={() => select(option.value)}
                aria-label={option.label || option.tooltip}
            >
                {#if option.icon}
                    <Icon name={option.icon} size={18} />
                {/if}
                {#if option.label}
                    <span class={responsiveLabels ? 'hidden xl:inline' : ''}>{option.label}</span>
                {/if}
            </button>
        </div>
    {/each}
</div>

<style>
    /* Override join defaults to allow gap */
    .join .join-item {
        border-radius: var(--mw-radius-xs);
    }
    .join {
        display: flex; /* Flexbox for better control than daisyui join sometimes */
    }
</style>
