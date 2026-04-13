<script lang="ts">
    /**
     * BrowseInput Component - /gallery Style
     * Refined input with icon and focus ring from /gallery ExportModal.
     */

    let {
        value = "",
        placeholder = "Select...",
        disabled = false,
        icon = "folder_open",
        onbrowse,
    }: {
        value?: string;
        placeholder?: string;
        disabled?: boolean;
        icon?: string;
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
        class="flex items-center w-full px-2 py-1.5 rounded-lg border border-base-300 bg-base-100 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20 transition-all"
        class:opacity-50={disabled}
    >
        <span class="material-symbols-rounded text-primary/70 text-sm ml-1"
            >{icon}</span
        >
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
            class="btn btn-sm btn-ghost bg-base-200 hover:bg-base-300 text-base-content/70 ml-2 px-4 rounded-md border-none font-normal"
            {disabled}
        >
            Browse
        </button>
    </div>
</div>
