<script lang="ts">
  import type { ManifestItem } from "./manifest";
  import { globPath } from "./manifest";
  import ComponentPreview from "./ComponentPreview.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  let { item, ready = false }: { item: ManifestItem; ready?: boolean } = $props();

  const path = $derived(globPath(item));
  const examples = $derived(item.examples ?? [{}]);
  let copied = $state(false);

  async function copyImport(): Promise<void> {
    try {
      await navigator.clipboard.writeText(item.importPath);
      copied = true;
      setTimeout(() => (copied = false), 1200);
    } catch {
      // clipboard unavailable — ignore
    }
  }
</script>

<section class="cd" id={"c-" + item.name}>
  <h3 class="cd-name">
    {item.name}
    {#if item.kind === "seed"}<span class="cd-tag cd-tag-seed">store-driven</span>{/if}
    {#if item.kind === "note"}<span class="cd-tag cd-tag-note">not mounted</span>{/if}
  </h3>

  <button class="cd-path" title="複製 import path" onclick={copyImport}>
    <Icon name={copied ? "circle-check" : "code"} size={12} />
    <code>{item.importPath}</code>
  </button>

  {#if item.blurb}<p class="cd-blurb">{item.blurb}</p>{/if}

  <div class="cd-preview">
    {#if item.kind === "note"}
      <div class="cd-note"><Icon name="info" size={14} /> <span>{item.note}</span></div>
    {:else if item.kind === "seed" && !ready}
      <span class="cd-dim">seeding…</span>
    {:else}
      {#each examples as ex, i (i)}
        <div class="cd-ex">
          <div class="cd-ex-render">
            <ComponentPreview {path} props={ex.props ?? {}} text={ex.text} />
          </div>
          {#if ex.label}<span class="cd-ex-label">{ex.label}</span>{/if}
        </div>
      {/each}
    {/if}
  </div>
</section>

<style>
  .cd {
    padding: 8px 0 4px;
    scroll-margin-top: 16px;
  }
  .cd-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 17px;
    font-weight: 700;
    color: var(--color-base-content);
  }
  .cd-tag {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 1px 6px;
    border-radius: 999px;
  }
  .cd-tag-seed {
    color: var(--color-accent, #7c6f64);
    background: color-mix(in oklch, var(--color-accent, #7c6f64) 16%, transparent);
  }
  .cd-tag-note {
    color: var(--mw-ink-3);
    background: color-mix(in oklch, var(--mw-ink-3, gray) 14%, transparent);
  }
  .cd-path {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-top: 3px;
    color: var(--mw-ink-3);
    padding: 1px 0;
  }
  .cd-path:hover {
    color: var(--color-base-content);
  }
  .cd-path code {
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .cd-blurb {
    margin: 6px 0 0;
    font-size: 13px;
    line-height: 1.6;
    color: var(--mw-ink-2, var(--color-base-content));
    max-width: 70ch;
  }
  .cd-preview {
    margin-top: 12px;
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 16px;
    padding: 18px;
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-md, 10px);
    background: var(--color-base-200);
    min-height: 56px;
  }
  .cd-ex {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    min-width: 0;
  }
  .cd-ex-label {
    font-size: 10px;
    color: var(--mw-ink-3);
  }
  .cd-note {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-style: italic;
    color: var(--mw-ink-3);
  }
  .cd-dim {
    color: var(--mw-ink-3);
    font-size: 12px;
  }
</style>
