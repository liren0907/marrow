<script lang="ts">
  // Gallery's own left nav — a collapsible tree: route → group label →
  // component anchor link. Self-contained (not the global workspace Sidebar).
  import type { ManifestRoute } from "./manifest";
  import Icon from "$lib/components/ui/Icon.svelte";

  let {
    routes,
    selected,
    onselect,
  }: {
    routes: ManifestRoute[];
    selected: string;
    onselect: (routeId: string, anchor?: string) => void;
  } = $props();
</script>

<aside class="gs">
  <div class="gs-brand">
    <Icon name="palette" size={15} />
    <strong>Component Gallery</strong>
  </div>
  <div class="gs-sub">DEV · /GALLERY</div>

  <nav class="gs-tree">
    {#each routes as r (r.id)}
      {@const open = r.id === selected}
      <div class="gs-route">
        <button class="gs-route-head" class:open onclick={() => onselect(r.id)}>
          <Icon name={open ? "chevron-down" : "chevron-right"} size={13} />
          <span>{r.title}</span>
        </button>
        {#if open}
          <div class="gs-children">
            {#each r.groups as g (g.label)}
              <div class="gs-grouplabel">{g.label}</div>
              {#each g.items as it (it.name)}
                <button class="gs-item" onclick={() => onselect(r.id, "c-" + it.name)}>
                  {it.name}
                </button>
              {/each}
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </nav>
</aside>

<style>
  .gs {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 260px;
    flex-shrink: 0;
    border-right: 1px solid var(--mw-rule);
    background: var(--color-base-200);
    overflow-y: auto;
    padding: 12px 10px;
  }
  .gs-brand {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 14px;
    color: var(--color-base-content);
  }
  .gs-sub {
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--mw-ink-3);
    margin: 2px 0 12px 22px;
  }
  .gs-route-head {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    padding: 5px 6px;
    border-radius: var(--mw-radius-xs);
    font-weight: 600;
    font-size: 13px;
    color: var(--color-base-content);
  }
  .gs-route-head:hover {
    background: color-mix(in oklch, var(--mw-accent) 7%, transparent);
  }
  .gs-children {
    margin: 2px 0 8px 13px;
    padding-left: 8px;
    border-left: 1px solid var(--mw-rule);
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .gs-grouplabel {
    font-size: 9.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--mw-ink-3);
    margin: 8px 0 2px 6px;
  }
  .gs-item {
    text-align: left;
    padding: 3px 8px;
    border-radius: var(--mw-radius-xs);
    font-size: 12px;
    color: var(--mw-ink-1, var(--color-base-content));
  }
  .gs-item:hover {
    background: color-mix(in oklch, var(--mw-accent) 9%, transparent);
    color: var(--color-base-content);
  }
</style>
