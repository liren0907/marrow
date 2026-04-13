<script lang="ts">
    let {
        type = "text",
        value = $bindable(""),
        size = "sm",
        variant = "bordered",
        readonly: isReadonly = false,
        disabled = false,
        placeholder = "",
        step = undefined,
        class: className = "",
        onchange,
        oninput,
        onkeydown,
    }: {
        type?: "text" | "number";
        value?: string | number;
        size?: "xs" | "sm" | "md";
        variant?: "bordered" | "ghost";
        readonly?: boolean;
        disabled?: boolean;
        placeholder?: string;
        step?: string;
        class?: string;
        onchange?: (e: Event) => void;
        oninput?: (e: Event) => void;
        onkeydown?: (e: KeyboardEvent) => void;
    } = $props();

    let sizeClass = $derived(size ? `input-${size}` : "");
    let variantClass = $derived(`input-${variant}`);
</script>

<input
    {type}
    bind:value
    class="input {variantClass} {sizeClass} {className}"
    readonly={isReadonly}
    {disabled}
    {placeholder}
    {step}
    {onchange}
    {oninput}
    {onkeydown}
/>
