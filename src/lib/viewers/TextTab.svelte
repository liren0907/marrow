<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { EditorState, Compartment, type Extension } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { basicSetup } from "codemirror";
  import type { Tab } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { readTextFile } from "$lib/workspace/tauri";
  import { showError } from "$lib/stores/toastStore.svelte";
  import { languageFor } from "./cm/languages";
  import { themeFor } from "./cm/theme";

  let { tab }: { tab: Tab } = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  let cancelled = false;
  let loadError = $state<string | null>(null);
  let lastHandledToken = 0;
  const themeCompartment = new Compartment();
  let themeObserver: MutationObserver | null = null;

  function currentTheme(): string {
    return document.documentElement.getAttribute("data-theme") ?? "light";
  }

  function buildExtensions(): Extension[] {
    return [
      basicSetup,
      EditorView.editable.of(false),
      EditorState.readOnly.of(true),
      languageFor(tab.path),
      themeCompartment.of(themeFor(currentTheme())),
    ];
  }

  function setContent(content: string) {
    if (!view) return;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: content },
    });
  }

  onMount(() => {
    (async () => {
      try {
        const result = await readTextFile(tab.path);
        if (cancelled) return;
        view = new EditorView({
          parent: host,
          state: EditorState.create({
            doc: result.content,
            extensions: buildExtensions(),
          }),
        });
        workspace.patchTab(tab.id, { lastKnownMtime: result.mtime });
        themeObserver = new MutationObserver(() => {
          if (!view) return;
          view.dispatch({
            effects: themeCompartment.reconfigure(themeFor(currentTheme())),
          });
        });
        themeObserver.observe(document.documentElement, {
          attributes: true,
          attributeFilter: ["data-theme"],
        });
      } catch (e) {
        loadError = e instanceof Error ? e.message : String(e);
      }
    })();
  });

  onDestroy(() => {
    cancelled = true;
    themeObserver?.disconnect();
    themeObserver = null;
    view?.destroy();
    view = null;
  });

  $effect(() => {
    const token = tab.reloadToken ?? 0;
    if (!view || token === lastHandledToken) return;
    lastHandledToken = token;
    (async () => {
      try {
        const result = await readTextFile(tab.path);
        setContent(result.content);
        workspace.patchTab(tab.id, { lastKnownMtime: result.mtime });
      } catch (e) {
        showError(
          `Failed to reload ${tab.title}: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    })();
  });
</script>

<div class="w-full h-full overflow-hidden">
  {#if loadError}
    <div class="p-6 text-error text-sm">Failed to load: {loadError}</div>
  {/if}
  <div bind:this={host} class="cm-host w-full h-full overflow-auto"></div>
</div>

<style>
  :global(.cm-host .cm-editor) {
    height: 100%;
    font-size: 13px;
  }
  :global(.cm-host .cm-scroller) {
    font-family: var(--font-mono, ui-monospace, monospace);
  }
</style>
