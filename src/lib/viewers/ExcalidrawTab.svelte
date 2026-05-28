<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { Tab } from "$lib/workspace/types";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { readTextFile, writeTextFile } from "$lib/workspace/tauri";
  import { registerTabSave, unregisterTabSave } from "$lib/workspace/tabRegistry.svelte";
  import { openConflict } from "$lib/conflict/conflictState.svelte";
  import { debounce } from "$lib/utils/debounce";
  import { editorSettings } from "$lib/settings/editorSettings.svelte";
  import { showError, showWarning } from "$lib/stores/toastStore.svelte";

  let { tab }: { tab: Tab } = $props();

  let host: HTMLDivElement;
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let lastHandledToken = 0;

  // React handles + Excalidraw imperative API must live OUTSIDE Svelte 5's
  // reactive proxy. CLAUDE.md rule #1: editor-like instances corrupt if
  // observed. These are plain `let` bindings on purpose.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let reactRoot: any = null;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let api: any = null;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let ExcalidrawComp: any = null;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let serializeAsJSON: any = null;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let createElement: any = null;

  let savedContent = "";
  let themeObserver: MutationObserver | null = null;
  let cancelled = false;

  function currentTheme(): "light" | "dark" {
    const t = document.documentElement.getAttribute("data-theme") ?? "light";
    return t === "dark" || t === "synthwave" || t === "marrow-pro-dark"
      ? "dark"
      : "light";
  }

  function serializeCurrent(): string {
    if (!api || !serializeAsJSON) return "";
    return serializeAsJSON(
      api.getSceneElements(),
      api.getAppState(),
      api.getFiles(),
      "local",
    );
  }

  async function save(): Promise<void> {
    const next = serializeCurrent();
    if (!next || next === savedContent) return;
    try {
      const result = await writeTextFile(tab.path, next, tab.lastKnownMtime);
      savedContent = next;
      workspace.patchTab(tab.id, {
        isDirty: false,
        lastKnownMtime: result.mtime,
        lastSavedTs: Date.now(),
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes("File changed on disk")) {
        void openConflict(tab, next);
      } else {
        showError(`Failed to save ${tab.title}: ${msg}`);
      }
    }
  }

  const debouncedSave = debounce(
    () => void save(),
    () => editorSettings.autosaveDebounceMs,
  );

  function saveNow(): Promise<void> {
    debouncedSave.cancel();
    return save();
  }

  function handleChange(): void {
    const next = serializeCurrent();
    if (!next) return;
    const dirty = next !== savedContent;
    if (dirty !== tab.isDirty) {
      workspace.patchTab(tab.id, { isDirty: dirty });
    }
    if (dirty) debouncedSave();
  }

  function renderReact(initialData: unknown): void {
    if (!reactRoot || !ExcalidrawComp || !createElement) return;
    reactRoot.render(
      createElement(ExcalidrawComp, {
        initialData,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        excalidrawAPI: (a: any) => {
          api = a;
        },
        onChange: handleChange,
        theme: currentTheme(),
      }),
    );
  }

  function applyTheme(): void {
    if (!reactRoot || !api) return;
    renderReact({
      elements: api.getSceneElements(),
      appState: api.getAppState(),
      files: api.getFiles(),
    });
  }

  onMount(() => {
    registerTabSave(tab.id, saveNow);
    (async () => {
      try {
        const [{ createRoot }, react, excalidraw] = await Promise.all([
          import("react-dom/client"),
          import("react"),
          import("@excalidraw/excalidraw"),
          import("@excalidraw/excalidraw/index.css"),
        ]);
        if (cancelled) return;

        createElement = react.createElement;
        ExcalidrawComp = excalidraw.Excalidraw;
        serializeAsJSON = excalidraw.serializeAsJSON;

        const result = await readTextFile(tab.path);
        if (cancelled) return;
        savedContent = result.content;
        workspace.patchTab(tab.id, { lastKnownMtime: result.mtime });

        let initialData: unknown = null;
        if (result.content.trim()) {
          try {
            initialData = JSON.parse(result.content);
          } catch (e) {
            console.warn("[excalidraw] invalid JSON, starting blank", e);
            initialData = null;
          }
        }

        reactRoot = createRoot(host);
        renderReact(initialData);

        themeObserver = new MutationObserver(() => applyTheme());
        themeObserver.observe(document.documentElement, {
          attributes: true,
          attributeFilter: ["data-theme"],
        });

        loaded = true;
      } catch (e) {
        loadError = e instanceof Error ? e.message : String(e);
      }
    })();
  });

  onDestroy(() => {
    cancelled = true;
    unregisterTabSave(tab.id);
    debouncedSave.cancel();
    themeObserver?.disconnect();
    themeObserver = null;
    reactRoot?.unmount();
    reactRoot = null;
    api = null;
  });

  $effect(() => {
    const token = tab.reloadToken ?? 0;
    if (!loaded || token === lastHandledToken) return;
    lastHandledToken = token;
    if (tab.isDirty) {
      showWarning(`${tab.title} changed on disk — still editing, not reloaded`);
      return;
    }
    (async () => {
      try {
        const result = await readTextFile(tab.path);
        savedContent = result.content;
        workspace.patchTab(tab.id, {
          isDirty: false,
          lastKnownMtime: result.mtime,
        });
        if (api && result.content.trim()) {
          try {
            const parsed = JSON.parse(result.content);
            api.updateScene({
              elements: parsed.elements ?? [],
              appState: parsed.appState ?? undefined,
            });
          } catch (e) {
            console.warn("[excalidraw] reload parse failed", e);
          }
        }
      } catch (e) {
        showError(
          `Failed to reload ${tab.title}: ${e instanceof Error ? e.message : String(e)}`,
        );
      }
    })();
  });
</script>

<div class="w-full h-full relative flex flex-col min-h-0">
  {#if loadError}
    <div class="p-6 text-error text-sm">Failed to load: {loadError}</div>
  {/if}
  <div bind:this={host} class="excalidraw-host flex-1 min-h-0"></div>
</div>

<style>
  .excalidraw-host {
    width: 100%;
    overflow: hidden;
  }
  :global(.excalidraw-host .excalidraw) {
    height: 100%;
  }
</style>
