<script lang="ts">
  import { onDestroy, untrack } from "svelte";
  import { EditorState, Compartment, type Extension } from "@codemirror/state";
  import { EditorView } from "@codemirror/view";
  import { basicSetup } from "codemirror";
  import { markdown as markdownLang } from "@codemirror/lang-markdown";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import type { Tab, FileMeta } from "$lib/workspace/types";
  import {
    classifyFile,
    basename,
    dirname,
    joinPath,
    isConvertible,
    CONVERTIBLE_EXTS,
  } from "$lib/workspace/fileKind";
  import {
    convertToMarkdown,
    convertHtmlToMarkdown,
    convertDocxToMarkdown,
    convertPptxToMarkdown,
    writeTextFile,
    writeBinaryFile,
    createDirectory,
    type ConvertAsset,
  } from "$lib/workspace/tauri";
  import { pdfToMarkdown } from "$lib/convert/pdfToMarkdown";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    getCached,
    setCached,
    type ConvertEngine,
  } from "./convertCache.svelte";
  import { showError, showSuccess } from "$lib/stores/toastStore.svelte";
  import { openNamePrompt } from "$lib/tree/namePromptState.svelte";
  import { themeFor } from "./cm/theme";
  import PdfTab from "./PdfTab.svelte";
  import ImageTab from "./ImageTab.svelte";
  import UnsupportedTab from "./UnsupportedTab.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";
  import { Button, ToggleButtonGroup } from "$lib/components/ui";

  let { tab }: { tab: Tab } = $props();

  // Extensions the Native engine can convert with no external runtime.
  const NATIVE_EXTS = ["pdf", "html", "htm", "docx", "pptx"];

  const extOf = (p: string): string =>
    (p.split(".").pop() ?? "").toLowerCase();
  const isNativeExt = (p: string): boolean => NATIVE_EXTS.includes(extOf(p));

  const isWorkspaceMode = $derived(tab.path === "marrow://convert");
  let internalSource = $state<string | null>(null);
  const sourcePath = $derived<string | null>(
    isWorkspaceMode ? internalSource : tab.path,
  );
  const sourceKind = $derived(
    sourcePath ? classifyFile(sourcePath) : "unsupported",
  );
  const sourceTab = $derived<Tab | null>(
    sourcePath
      ? { ...tab, path: sourcePath, kind: sourceKind, title: basename(sourcePath) }
      : null,
  );

  // ─── Conversion engine ──────────────────────────────────────────────────
  // "native"     — built-in converters, zero external deps (pdf via pdfjs;
  //                html/docx/pptx via the marrow-convert Rust crate).
  // "markitdown" — `uvx markitdown`, widest format support, needs `uv`.
  const ENGINE_STORAGE_KEY = "marrow.convert.engine";

  function loadStoredEngine(): ConvertEngine | null {
    if (typeof localStorage === "undefined") return null;
    const raw = localStorage.getItem(ENGINE_STORAGE_KEY);
    return raw === "native" || raw === "markitdown" ? raw : null;
  }

  function storeEngine(e: ConvertEngine): void {
    if (typeof localStorage === "undefined") return;
    try {
      localStorage.setItem(ENGINE_STORAGE_KEY, e);
    } catch {
      // ignore — private mode / quota
    }
  }

  /** Smart initial engine: a known source the Native engine can't handle
   * forces markitdown; otherwise honor the user's last choice, else native. */
  function defaultEngineFor(path: string | null): ConvertEngine {
    if (path && !isNativeExt(path)) return "markitdown";
    return loadStoredEngine() ?? "native";
  }

  // One-time smart default; `untrack` since the initializer deliberately
  // captures only the initial source/mode, not a reactive dependency.
  let engine = $state<ConvertEngine>(
    untrack(() => defaultEngineFor(isWorkspaceMode ? null : tab.path)),
  );

  function setEngine(next: ConvertEngine): void {
    if (engine === next) return;
    engine = next;
    storeEngine(next);
  }

  let status = $state<"idle" | "loading" | "ready" | "error">("idle");
  let markdown = $state("");
  /** Sidecar assets for the current conversion (DOCX/PPTX pictures today).
   * Held in memory until the user saves; written under
   * `<saveDir>/attachments/` next to the .md. */
  let assets = $state<ConvertAsset[]>([]);
  let errorMessage = $state("");
  let slowHint = $state(false);
  let slowTimer: ReturnType<typeof setTimeout> | null = null;

  let filterQuery = $state("");
  const convertibleFiles = $derived.by(() => {
    const q = filterQuery.trim().toLowerCase();
    const all = workspace.fileIndex.filter((f) => isConvertible(f.path));
    if (!q) return all;
    return all.filter((f) => f.name.toLowerCase().includes(q));
  });

  /** A source is picked but the Native engine cannot handle its extension —
   * the markdown pane shows a one-click "switch to markitdown" hint. */
  const engineMismatch = $derived(
    !!sourcePath && engine === "native" && !isNativeExt(sourcePath),
  );

  function iconForExt(path: string): IconName {
    const ext = extOf(path);
    if (ext === "pdf" || ext === "docx" || ext === "pptx") return "file-text";
    if (ext === "xlsx" || ext === "xls" || ext === "csv") return "file-text";
    if (ext === "html" || ext === "htm" || ext === "xml") return "file-code";
    if (ext === "json" || ext === "ipynb") return "file-code";
    return "file";
  }

  function relPath(path: string): string {
    const root = workspace.info?.root;
    if (!root) return path;
    if (path.startsWith(root)) {
      const rel = path.slice(root.length).replace(/^[/\\]+/, "");
      return rel || basename(path);
    }
    return path;
  }

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  let cancelled = false;
  /** Guard against redundant re-conversions. Composite of engine + path
   * because the same file converts differently per engine. */
  let lastConvertedKey: string | null = null;
  const convKey = (e: ConvertEngine, p: string): string =>
    JSON.stringify([e, p]);
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
      markdownLang(),
      themeCompartment.of(themeFor(currentTheme())),
    ];
  }

  function mountEditorIfNeeded(content: string) {
    if (!host) return;
    if (view) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: content },
      });
      return;
    }
    view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: content,
        extensions: buildExtensions(),
      }),
    });
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
  }

  async function runConvert(path: string, force = false) {
    // Capture the engine at call time so a mid-flight toggle is well-defined.
    const engineAtStart = engine;
    if (!force) {
      const cached = getCached(engineAtStart, path);
      if (cached !== null) {
        markdown = cached.markdown;
        assets = cached.assets;
        status = "ready";
        queueMicrotask(() => mountEditorIfNeeded(cached.markdown));
        return;
      }
    }
    status = "loading";
    errorMessage = "";
    slowHint = false;
    if (slowTimer) clearTimeout(slowTimer);
    const ext = extOf(path);
    if (engineAtStart === "markitdown") {
      slowTimer = setTimeout(() => {
        if (!cancelled) slowHint = true;
      }, 4000);
    }
    try {
      let resultMd: string;
      let resultAssets: ConvertAsset[] = [];
      if (engineAtStart === "native") {
        if (ext === "pdf") {
          resultMd = await pdfToMarkdown(path);
        } else if (ext === "html" || ext === "htm") {
          resultMd = await convertHtmlToMarkdown(path);
        } else if (ext === "docx") {
          const r = await convertDocxToMarkdown(path);
          resultMd = r.markdown;
          resultAssets = r.assets;
        } else if (ext === "pptx") {
          const r = await convertPptxToMarkdown(path);
          resultMd = r.markdown;
          resultAssets = r.assets;
        } else {
          throw new Error(
            `The Native engine does not support .${ext} files. Switch to markitdown.`,
          );
        }
      } else {
        resultMd = await convertToMarkdown(path);
      }
      if (cancelled || sourcePath !== path || engine !== engineAtStart) return;
      markdown = resultMd;
      assets = resultAssets;
      setCached(engineAtStart, path, resultMd, resultAssets);
      status = "ready";
      queueMicrotask(() => mountEditorIfNeeded(resultMd));
    } catch (e) {
      if (cancelled || sourcePath !== path || engine !== engineAtStart) return;
      errorMessage = e instanceof Error ? e.message : String(e);
      status = "error";
    } finally {
      if (slowTimer) {
        clearTimeout(slowTimer);
        slowTimer = null;
      }
      slowHint = false;
    }
  }

  function handleRetry() {
    if (sourcePath) void runConvert(sourcePath, true);
  }

  function handleChangeSource() {
    internalSource = null;
    markdown = "";
    assets = [];
    errorMessage = "";
    status = "idle";
    lastConvertedKey = null;
  }

  async function handleBrowseExternal() {
    try {
      const picked = await openDialog({
        multiple: false,
        filters: [
          { name: "Convertible files", extensions: [...CONVERTIBLE_EXTS] },
        ],
      });
      if (typeof picked === "string") {
        internalSource = picked;
      }
    } catch (e) {
      showError(
        `Failed to open file dialog: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }

  function handlePick(f: FileMeta) {
    internalSource = f.path;
  }

  function handleSave() {
    if (status !== "ready" || !sourcePath) return;
    const src = sourcePath;
    const suggested = basename(src).replace(/\.[^.]+$/, "") + ".md";
    const root = workspace.info?.root;
    const inWorkspace = root ? src.startsWith(root) : false;
    // For workspace-view mode picking external files, default to workspace
    // root so the saved .md lands somewhere visible in the file tree.
    const defaultDir =
      isWorkspaceMode && !inWorkspace && root ? root : dirname(src);
    const contents = markdown;
    const assetsAtSave = assets;
    openNamePrompt({
      title: "Save as Markdown",
      initial: suggested,
      confirmLabel: "Save",
      onConfirm: async (name) => {
        const trimmed = name.trim();
        if (!trimmed) return;
        const finalName = trimmed.toLowerCase().endsWith(".md")
          ? trimmed
          : `${trimmed}.md`;
        const target = joinPath(defaultDir, finalName);
        try {
          await writeTextFile(target, contents);
          if (assetsAtSave.length > 0) {
            const attachDir = joinPath(defaultDir, "attachments");
            try {
              await createDirectory(attachDir);
            } catch (e) {
              // Tolerate "Already exists" from the backend's create_directory.
              const msg = e instanceof Error ? e.message : String(e);
              if (!/already exists/i.test(msg)) throw e;
            }
            for (const a of assetsAtSave) {
              const bytes = decodeBase64(a.bytes_b64);
              await writeBinaryFile(joinPath(attachDir, a.name), bytes);
            }
          }
          showSuccess(
            assetsAtSave.length > 0
              ? `Saved ${finalName} (+${assetsAtSave.length} attachment${assetsAtSave.length === 1 ? "" : "s"})`
              : `Saved ${finalName}`,
          );
          void workspace.refreshFileIndex();
          workspace.openFile(target);
        } catch (e) {
          showError(
            `Failed to save: ${e instanceof Error ? e.message : String(e)}`,
          );
        }
      },
    });
  }

  function decodeBase64(b64: string): Uint8Array {
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  }

  const isMissingUv = $derived(
    errorMessage.toLowerCase().includes("failed to spawn uvx"),
  );

  onDestroy(() => {
    cancelled = true;
    if (slowTimer) clearTimeout(slowTimer);
    themeObserver?.disconnect();
    themeObserver = null;
    view?.destroy();
    view = null;
  });

  // Convert when the source OR the engine changes — drives both the initial
  // pinned-mode conversion and every workspace-mode pick / engine toggle.
  $effect(() => {
    const path = sourcePath;
    const e = engine;
    if (!path) return;
    // Native engine can't handle this ext — skip (runConvert would only
    // throw); the inline mismatch hint offers a one-click switch instead.
    if (e === "native" && !isNativeExt(path)) {
      lastConvertedKey = convKey(e, path);
      return;
    }
    const key = convKey(e, path);
    if (key === lastConvertedKey) return;
    lastConvertedKey = key;
    void runConvert(path, false);
  });

  // Keep the tab title in sync with current source (workspace mode only).
  $effect(() => {
    if (!isWorkspaceMode) return;
    const title = sourcePath ? `Convert: ${basename(sourcePath)}` : "Convert";
    if (tab.title !== title) workspace.patchTab(tab.id, { title });
  });

  // External reload token — re-run conversion with current source + engine.
  $effect(() => {
    const token = tab.reloadToken ?? 0;
    if (token === 0 || !sourcePath) return;
    if (engine === "native" && !isNativeExt(sourcePath)) return;
    void runConvert(sourcePath, true);
  });
</script>

<div class="convert-tab w-full h-full flex flex-col bg-base-100">
  <div class="convert-header flex items-center justify-between px-3 py-2 border-b border-base-300 min-h-[40px]">
    <div class="text-xs text-base-content/60 truncate flex items-center gap-1">
      {#if sourcePath}
        <span class="truncate">{basename(sourcePath)}</span>
        <span class="text-base-content/40">→ Markdown</span>
      {:else}
        <span>Convert to Markdown</span>
      {/if}
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <ToggleButtonGroup
        size="sm"
        tooltipPosition="bottom"
        value={engine}
        onchange={setEngine}
        options={[
          {
            value: "native",
            label: "Native",
            tooltip: "Built-in converters · pdf, html, docx, pptx · no setup",
          },
          {
            value: "markitdown",
            label: "markitdown",
            tooltip: "uvx markitdown · all formats · needs uv installed",
          },
        ]}
      />
      {#if isWorkspaceMode && sourcePath}
        <Button size="xs" onclick={handleChangeSource}>
          <Icon name="arrow-left" size={14} class="mr-1" />
          Change
        </Button>
      {/if}
      {#if status === "ready" && !engineMismatch}
        <Button size="xs" onclick={handleSave}>
          <Icon name="file-plus" size={14} class="mr-1" />
          Save as .md
        </Button>
      {/if}
    </div>
  </div>

  <div class="flex-1 min-h-0 grid grid-cols-2 divide-x divide-base-300">
    <div class="source-pane min-w-0 relative overflow-hidden">
      {#if isWorkspaceMode && !sourcePath}
        <div class="picker p-4 h-full flex flex-col gap-3 min-h-0">
          <div class="text-xs text-base-content/60">
            Pick a file to convert to Markdown
          </div>
          <input
            type="text"
            class="input input-sm w-full"
            placeholder="Filter workspace files..."
            bind:value={filterQuery}
          />
          <div class="flex-1 min-h-0 overflow-auto flex flex-col gap-0.5 pr-1">
            {#each convertibleFiles as f (f.path)}
              <button
                type="button"
                class="text-left px-2 py-1.5 rounded hover:bg-base-200 flex items-center gap-2 min-w-0"
                onclick={() => handlePick(f)}
              >
                <Icon name={iconForExt(f.path)} size={14} class="shrink-0 text-base-content/60" />
                <span class="text-sm truncate">{f.name}</span>
                <span class="text-xs text-base-content/40 ml-auto truncate">
                  {relPath(f.path)}
                </span>
              </button>
            {:else}
              <div class="text-xs text-base-content/40 p-2">
                {filterQuery
                  ? "No matches."
                  : "No convertible files in workspace."}
              </div>
            {/each}
          </div>
          <div class="border-t border-base-300 pt-3">
            <Button size="sm" onclick={handleBrowseExternal}>
              <Icon name="external-link" size={14} class="mr-1" />
              Browse external file…
            </Button>
          </div>
        </div>
      {:else if sourceTab && sourceKind === "pdf"}
        <PdfTab tab={sourceTab} />
      {:else if sourceTab && sourceKind === "image"}
        <ImageTab tab={sourceTab} />
      {:else if sourceTab}
        <UnsupportedTab tab={sourceTab} />
      {/if}
    </div>

    <div class="md-pane min-w-0 relative overflow-hidden">
      {#if isWorkspaceMode && !sourcePath}
        <div class="w-full h-full flex items-center justify-center text-xs text-base-content/40">
          Pick a source file on the left to see the Markdown here.
        </div>
      {:else if engineMismatch}
        <div class="w-full h-full flex items-center justify-center p-6">
          <div class="max-w-sm flex flex-col gap-3 items-start">
            <span class="text-sm font-semibold">
              Native engine can't convert this file
            </span>
            <p class="text-xs text-base-content/60 leading-relaxed">
              The Native engine handles PDF, HTML, DOCX and PPTX only.
              <code class="bg-base-200 px-1 rounded">.{extOf(sourcePath ?? "")}</code>
              files need the markitdown engine.
            </p>
            <Button size="sm" onclick={() => setEngine("markitdown")}>
              <Icon name="flask-conical" size={14} class="mr-1" />
              Switch to markitdown
            </Button>
          </div>
        </div>
      {:else if status === "loading"}
        <div class="w-full h-full flex flex-col items-center justify-center gap-3 text-base-content/60 text-sm">
          <span class="loading loading-spinner loading-md"></span>
          <div>Converting to Markdown…</div>
          {#if slowHint}
            <div class="text-xs text-base-content/40 max-w-xs text-center px-4">
              First-time run downloads markitdown dependencies. This happens only once.
            </div>
          {/if}
        </div>
      {:else if status === "error"}
        <div class="w-full h-full flex items-center justify-center p-6">
          <div class="max-w-md flex flex-col gap-3 items-start">
            <div class="flex items-center gap-2 text-error">
              <Icon name="triangle-alert" size={18} />
              <span class="text-sm font-semibold">Conversion failed</span>
            </div>
            <pre class="text-xs whitespace-pre-wrap break-words text-base-content/70 bg-base-200 p-3 rounded max-h-48 overflow-auto w-full">{errorMessage}</pre>
            {#if isMissingUv && engine === "markitdown"}
              <div class="text-xs text-base-content/60">
                Install <code class="bg-base-200 px-1 rounded">uv</code> with:
                <pre class="text-xs bg-base-200 p-2 mt-1 rounded">curl -LsSf https://astral.sh/uv/install.sh | sh</pre>
              </div>
            {/if}
            <Button size="sm" onclick={handleRetry}>
              <Icon name="rotate-ccw" size={14} class="mr-1" />
              Retry
            </Button>
          </div>
        </div>
      {/if}
      <div
        bind:this={host}
        class="cm-host w-full h-full overflow-auto"
        class:hidden={status !== "ready" || engineMismatch}
      ></div>
    </div>
  </div>
</div>

<style>
  :global(.convert-tab .cm-host .cm-editor) {
    height: 100%;
    font-size: 13px;
  }
  :global(.convert-tab .cm-host .cm-scroller) {
    font-family: var(--font-mono, ui-monospace, monospace);
  }
</style>
