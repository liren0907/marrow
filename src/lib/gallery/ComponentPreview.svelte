<script lang="ts">
  // Renders ONE live example, loaded lazily by path. Wrapped in
  // <svelte:boundary> (Svelte 5.3+) so a misbehaving preview shows an inline
  // error instead of taking down the page. `props` may be a thunk so seeded
  // examples can read live workspace data at render time. Svelte 5 dynamic tag.
  import { LOADERS } from "./discovery";

  let {
    path,
    props = {},
    text = undefined,
  }: {
    path: string;
    props?: Record<string, unknown> | (() => Record<string, unknown>);
    text?: string | undefined;
  } = $props();

  const loader = $derived(LOADERS[path]);
  const resolvedProps = $derived(typeof props === "function" ? props() : props);
</script>

<svelte:boundary>
  {#if loader}
    {#await loader()}
      <span class="gp-dim">…</span>
    {:then mod}
      {@const Comp = mod.default}
      {#if text != null}
        <Comp {...resolvedProps}>{text}</Comp>
      {:else}
        <Comp {...resolvedProps} />
      {/if}
    {/await}
  {:else}
    <span class="gp-dim">no loader: {path}</span>
  {/if}

  {#snippet failed(error)}
    <span class="gp-err">⚠ {error instanceof Error ? error.message : String(error)}</span>
  {/snippet}
</svelte:boundary>

<style>
  .gp-dim {
    color: var(--mw-ink-3);
    font-size: 11px;
  }
  .gp-err {
    color: var(--color-error);
    font-size: 11px;
    word-break: break-word;
  }
</style>
