<script lang="ts">
  // Whisper-style icon button for chrome / meta-header contexts.
  //
  // IconButton (DaisyUI btn-based) is too visually heavy when sitting next
  // to mw-meta labels — its 32px chunky frame "shouts" while the labels
  // whisper. This component is the calm counterpart: 14px icon, transparent
  // background, ink color tokens, subtle hover, accent-color (no fill) for
  // active state. Use it for the editor meta header's view-control cluster
  // and similar low-volume toolbars.
  //
  // For the loud toolbar buttons (graph view top bar, sidebar primary
  // actions, etc.) keep using IconButton — the contexts are different.

  import Icon, { type IconName } from "./Icon.svelte";

  let {
    icon,
    tooltip = undefined,
    active = false,
    ariaLabel = undefined,
    onclick,
  }: {
    icon: IconName;
    tooltip?: string | undefined;
    active?: boolean;
    ariaLabel?: string | undefined;
    onclick?: (event: MouseEvent) => void;
  } = $props();
</script>

<div class="tooltip tooltip-bottom" data-tip={tooltip}>
  <button
    type="button"
    class="meta-icon-btn"
    class:active
    aria-label={ariaLabel ?? tooltip}
    onclick={(e) => onclick?.(e)}
  >
    <Icon name={icon} size={14} />
  </button>
</div>

<style>
  .meta-icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--mw-ink-3);
    cursor: pointer;
    transition:
      color 0.12s ease,
      background-color 0.12s ease;
  }
  .meta-icon-btn:hover {
    color: var(--mw-ink-1);
    background-color: color-mix(
      in oklch,
      var(--color-base-content) 6%,
      transparent
    );
  }
  .meta-icon-btn.active {
    /* Color-only active state — no filled background, so it stays in the
       same visual register as the rest of the meta row. */
    color: var(--mw-accent);
  }
  .meta-icon-btn.active:hover {
    background-color: color-mix(
      in oklch,
      var(--mw-accent) 10%,
      transparent
    );
  }
  .meta-icon-btn:focus-visible {
    outline: 2px solid color-mix(in oklch, var(--mw-accent) 60%, transparent);
    outline-offset: 1px;
  }
</style>
