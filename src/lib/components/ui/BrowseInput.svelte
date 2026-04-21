<script lang="ts">
    /**
     * BrowseInput Component - /gallery Style
     * Refined input with icon and focus ring from /gallery ExportModal.
     */

    import Icon, { type IconName } from "./Icon.svelte";

    let {
        value = "",
        placeholder = "Select...",
        disabled = false,
        icon = "folder-open",
        onbrowse,
    }: {
        value?: string;
        placeholder?: string;
        disabled?: boolean;
        icon?: IconName;
        onbrowse?: () => void;
    } = $props();

    function handleBrowseClick() {
        if (!disabled) {
            onbrowse?.();
        }
    }
</script>

<div class="form-control">
    <div
        class="flex items-center w-full px-2 py-1.5 rounded-[var(--mw-radius-sm)] border border-base-300 bg-base-100 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20 transition-all"
        class:opacity-50={disabled}
    >
        <Icon name={icon} size={14} class="text-primary/70 ml-1" />
        <input
            type="text"
            {value}
            readonly
            {placeholder}
            class="input input-ghost w-full h-8 text-xs focus:outline-none border-none bg-transparent px-2"
            {disabled}
        />
        <button
            type="button"
            onclick={handleBrowseClick}
            class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 ml-2 px-4 rounded-[var(--mw-radius-sm)] border-none font-normal"
            {disabled}
        >
            Browse
        </button>
    </div>
</div>
