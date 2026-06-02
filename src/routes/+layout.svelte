<script lang="ts">
  // Minimal ROOT layout — applies to every route (the (app) workspace shell
  // AND the dev-only /gallery). It only owns "document appearance" concerns:
  // global CSS, theme, accent, fonts, prism highlight theme. The heavy
  // workspace chrome + runtime listeners live in (app)/+layout.svelte so that
  // /gallery can render clean, without ActivityBar/Sidebar/modals/fs-watchers.
  import "$lib/polyfills/readableStreamAsyncIterator";
  import { onMount } from "svelte";
  import { initAccent } from "$lib/settings/accentState.svelte";
  import { initAppearanceFonts } from "$lib/settings/appearanceSettings.svelte";
  import { initPrismTheme } from "$lib/settings/prismThemeLoader";
  import "../app.css";
  import "katex/dist/katex.min.css";

  let { children } = $props();

  onMount(() => {
    const legacyMap: Record<string, string> = {
      light: "marrow-pro-light",
      dark: "marrow-pro-dark",
    };
    const stored = localStorage.getItem("theme");
    const savedTheme = stored ? (legacyMap[stored] ?? stored) : "marrow-pro-light";
    document.documentElement.setAttribute("data-theme", savedTheme);
    if (stored && legacyMap[stored]) localStorage.setItem("theme", savedTheme);

    initAccent();
    initAppearanceFonts();
    initPrismTheme();
  });
</script>

{@render children()}
