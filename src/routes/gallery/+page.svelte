<script lang="ts">
  // Dev-only component catalog, organised by ROUTE → semantic GROUP → component
  // (manifest-driven, doc-style). Lives OUTSIDE the (app) route group so it gets
  // only the minimal root layout (CSS + theme) — no workspace chrome. Store-
  // driven components are rendered live after a dev-only seed of the devmock
  // demo workspace (no Tauri). Only the SELECTED route is mounted at a time, so
  // we never mount every heavy component at once.
  import { onMount, tick } from "svelte";
  import { MANIFEST } from "$lib/gallery/manifest";
  import GallerySidebar from "$lib/gallery/GallerySidebar.svelte";
  import ComponentDoc from "$lib/gallery/ComponentDoc.svelte";
  import { seedWorkspace } from "$lib/gallery/fixtures";

  const routes = MANIFEST;
  let selected = $state(routes[0]?.id ?? "");
  let ready = $state(false);
  const route = $derived(routes.find((r) => r.id === selected) ?? routes[0]);

  const THEMES = [
    "marrow-pro-light", "marrow-pro-dark", "marrow-light",
    "light", "dark", "cupcake", "corporate",
  ];
  let theme = $state("marrow-pro-light");

  function applyTheme(t: string): void {
    theme = t;
    document.documentElement.setAttribute("data-theme", t);
  }

  async function onselect(routeId: string, anchor?: string): Promise<void> {
    if (routeId !== selected) {
      selected = routeId;
      await tick();
    }
    if (anchor) {
      document.getElementById(anchor)?.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  }

  onMount(async () => {
    theme = document.documentElement.getAttribute("data-theme") ?? "marrow-pro-light";
    try {
      await seedWorkspace();
    } catch (e) {
      console.warn("[gallery] seed failed", e);
    }
    ready = true;
  });
</script>

<svelte:head><title>Gallery — Marrow</title></svelte:head>

{#if import.meta.env.DEV}
  <div class="gallery">
    <GallerySidebar {routes} {selected} {onselect} />

    <main class="gallery-main">
      <header class="gx-head">
        <div>
          <h1 class="gx-title">Component Gallery</h1>
          <p class="gx-intro">
            依視圖(route)分類列出 Marrow 用到的 UI 元件。純 prop-driven 用範例 props 呈現,
            store-driven 由 dev-only seed(devmock 假 workspace)餵入。<strong>Dev tool only — 不發任何 Tauri 請求。</strong>
          </p>
        </div>
        <label class="gx-theme">
          <span class="mw-meta">theme</span>
          <select value={theme} onchange={(e) => applyTheme(e.currentTarget.value)}>
            {#each THEMES as t (t)}<option value={t}>{t}</option>{/each}
          </select>
        </label>
      </header>

      {#if route}
        <section class="gx-route">
          <h2 class="gx-route-title">
            {route.title}
            {#if route.chip}<code class="gx-chip">{route.chip}</code>{/if}
          </h2>
          <p class="gx-route-blurb">{route.blurb}</p>

          {#each route.groups as g (g.label)}
            <h2 class="gx-group rule-{g.accent}">{g.label}</h2>
            {#each g.items as it (it.name)}
              <ComponentDoc item={it} {ready} />
            {/each}
          {/each}
        </section>
      {/if}
    </main>
  </div>
{:else}
  <div class="gallery-prod">Component gallery is a dev-only tool.</div>
{/if}

<style>
  .gallery {
    height: 100vh;
    display: flex;
    background: var(--color-base-100);
    color: var(--color-base-content);
    overflow: hidden;
  }
  .gallery-main {
    flex: 1;
    overflow-y: auto;
    padding: 28px 40px 80px;
    min-width: 0;
  }
  .gx-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--mw-rule);
  }
  .gx-title {
    font-size: 24px;
    font-weight: 700;
  }
  .gx-intro {
    margin-top: 6px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--mw-ink-2, var(--color-base-content));
    max-width: 80ch;
  }
  .gx-theme {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
  .gx-theme select {
    font-size: 12px;
    padding: 3px 6px;
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    background: var(--color-base-100);
    color: var(--color-base-content);
  }
  .gx-route-title {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 26px;
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .gx-chip {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 500;
    padding: 2px 7px;
    border-radius: var(--mw-radius-sm);
    background: var(--color-base-300, var(--color-base-200));
    color: var(--mw-ink-2, var(--color-base-content));
  }
  .gx-route-blurb {
    margin-top: 6px;
    font-size: 13px;
    line-height: 1.6;
    color: var(--mw-ink-3);
    max-width: 80ch;
  }
  .gx-group {
    margin: 34px 0 6px;
    padding-bottom: 8px;
    font-size: 13px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-bottom: 2px solid var(--mw-rule);
  }
  .rule-primary {
    border-bottom-color: var(--color-primary, var(--mw-accent));
    color: var(--color-primary, var(--mw-accent));
  }
  .rule-accent {
    border-bottom-color: var(--color-accent, var(--mw-accent));
    color: var(--color-accent, var(--mw-accent));
  }
  .rule-secondary {
    border-bottom-color: var(--color-secondary, var(--mw-accent));
    color: var(--color-secondary, var(--mw-accent));
  }
  .rule-info {
    border-bottom-color: var(--color-info, var(--mw-accent));
    color: var(--color-info, var(--mw-accent));
  }
  .gallery-prod {
    padding: 40px;
    text-align: center;
    color: var(--mw-ink-3);
  }
</style>
