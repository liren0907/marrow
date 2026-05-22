<script lang="ts">
  // CodeMirror 6 plain-text markdown editor — the "raw" / source-code view
  // counterpart to MilkdownEditor. Same external interface (initial in,
  // onChange string out) so MarkdownTab can swap between them with a single
  // conditional, without touching the autosave / dirty / reload pipeline.
  //
  // EditorView lives in a plain `let` (NOT $state) — Svelte 5's deep proxy
  // would corrupt CodeMirror's internal handles. See CLAUDE.md rule #1.

  import { onDestroy, onMount } from "svelte";
  import { EditorState, Compartment, type Extension } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { basicSetup } from "codemirror";
  import { markdown } from "@codemirror/lang-markdown";
  import { themeFor } from "$lib/viewers/cm/theme";
  import { registerCmApplyHook } from "$lib/settings/appearanceSettings.svelte";

  let {
    initial,
    onChange,
  }: {
    initial: string;
    onChange: (md: string) => void;
  } = $props();

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  // Set right before we dispatch our own programmatic doc replacements so
  // updateListener can ignore the resulting docChanged event. Protects us
  // from echoing changes back through onChange (would mark dirty falsely).
  // Currently only mount/unmount touch the doc, so this is unused — kept
  // as a hook for future programmatic content swaps (e.g. external reload
  // while in raw mode if we route reloads through here later).
  let suppressNextChange = false;
  const themeCompartment = new Compartment();
  let themeObserver: MutationObserver | null = null;
  let unregisterCmHook: (() => void) | null = null;

  function currentTheme(): string {
    return document.documentElement.getAttribute("data-theme") ?? "light";
  }

  function buildExtensions(): Extension[] {
    return [
      basicSetup,
      markdown(),
      themeCompartment.of(themeFor(currentTheme())),
      EditorView.updateListener.of((u) => {
        if (!u.docChanged) return;
        if (suppressNextChange) {
          suppressNextChange = false;
          return;
        }
        onChange(u.state.doc.toString());
      }),
    ];
  }

  onMount(() => {
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: initial,
        extensions: buildExtensions(),
      }),
    });

    // React to chrome-level theme attribute swaps (DaisyUI [data-theme]).
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

    // React to Settings → Appearance → CodeMirror theme changes.
    unregisterCmHook = registerCmApplyHook(() => {
      if (!view) return;
      view.dispatch({
        effects: themeCompartment.reconfigure(themeFor(currentTheme())),
      });
    });
  });

  onDestroy(() => {
    themeObserver?.disconnect();
    themeObserver = null;
    unregisterCmHook?.();
    unregisterCmHook = null;
    view?.destroy();
    view = null;
  });

  void suppressNextChange; // silence "declared but unused" until we wire it up
</script>

<div bind:this={host} class="md-raw-host w-full h-full overflow-auto"></div>

<style>
  :global(.md-raw-host .cm-editor) {
    height: 100%;
    /* Mirror the Milkdown view's reading column + body type so toggling
       pretty↔raw doesn't shift the visual reading position. font-family
       stays monospace — raw mode is a source view by design. */
    max-width: var(--mw-editor-max-width, 46rem);
    margin: 0 auto;
    font-size: var(--mw-editor-font-size, 16px);
  }
  :global(.md-raw-host .cm-scroller) {
    font-family: var(--font-mono, ui-monospace, monospace);
    padding: 1rem 1.25rem 6rem 1.25rem;
  }
  :global(.md-raw-host .cm-content) {
    line-height: var(--mw-editor-line-height, 1.65);
  }
</style>
