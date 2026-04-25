<script lang="ts">
  import { onDestroy, onMount } from "svelte";
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
  } from "$lib/workspace/fileKind";
  import {
    convertToMarkdown,
    convertHtmlToMarkdown,
    convertDocxToMarkdown,
    convertPptxToMarkdown,
    writeTextFile,
  } from "$lib/workspace/tauri";
  import { pdfToMarkdown } from "$lib/convert/pdfToMarkdown";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { getCached, setCached } from "./convertCache.svelte";
  import { showError, showSuccess } from "$lib/stores/toastStore.svelte";
  import { openNamePrompt } from "$lib/tree/namePromptState.svelte";
  import { themeFor } from "./cm/theme";
  import PdfTab from "./PdfTab.svelte";
  import ImageTab from "./ImageTab.svelte";
  import UnsupportedTab from "./UnsupportedTab.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";
  import { Button } from "$lib/components/ui";

  let { tab }: { tab: Tab } = $props();

  const NATIVE_EXTS = ["pdf", "html", "htm", "docx", "pptx"];

  const isNativeMode = $derived(tab.path === "marrow://convert-native");
  const isWorkspaceMode = $derived(
    tab.path === "marrow://convert" || tab.path === "marrow://convert-native",
  );
  let internalSource = $state<string | null>(null);
  const sourcePath = $derived<string | null>(
    isWorkspaceMode ? internalSource : tab.path,
  );
  const sourceKind = $derived(sourcePath ? classifyFile(sourcePath) : "unsupported");
  const sourceTab = $derived<Tab | null>(
    sourcePath
      ? { ...tab, path: sourcePath, kind: sourceKind, title: basename(sourcePath) }
      : null,
  );

  let status = $state<"idle" | "loading" | "ready" | "error">("idle");
  let markdown = $state("");
  let errorMessage = $state("");
  let slowHint = $state(false);
  let slowTimer: ReturnType<typeof setTimeout> | null = null;

  let filterQuery = $state("");
  const convertibleFiles = $derived.by(() => {
    const q = filterQuery.trim().toLowerCase();
    const predicate = isNativeMode
      ? (p: string) => {
          const ext = (p.split(".").pop() ?? "").toLowerCase();
          return NATIVE_EXTS.includes(ext);
        }
      : isConvertible;
    const all = workspace.fileIndex.filter((f) => predicate(f.path));
    if (!q) return all;
    return all.filter((f) => f.name.toLowerCase().includes(q));
  });

  function iconForExt(path: string): IconName {
    const ext = (path.split(".").pop() ?? "").toLowerCase();
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
  let lastConvertedPath: string | null = null;
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
    if (!force) {
      const cached = getCached(path);
      if (cached !== null) {
        markdown = cached;
        status = "ready";
        queueMicrotask(() => mountEditorIfNeeded(cached));
        return;
      }
    }
    status = "loading";
    errorMessage = "";
    slowHint = false;
    if (slowTimer) clearTimeout(slowTimer);
    const ext = (path.split(".").pop() ?? "").toLowerCase();
    const isNativeExt = NATIVE_EXTS.includes(ext);
    const useMarkitdown = !isNativeExt;
    if (useMarkitdown && !isNativeMode) {
      slowTimer = setTimeout(() => {
        if (!cancelled) slowHint = true;
      }, 4000);
    }
    try {
      let result: string;
      if (ext === "pdf") {
        result = await pdfToMarkdown(path);
      } else if (ext === "html" || ext === "htm") {
        result = await convertHtmlToMarkdown(path);
      } else if (ext === "docx") {
        result = await convertDocxToMarkdown(path);
      } else if (ext === "pptx") {
        result = await convertPptxToMarkdown(path);
      } else if (isNativeMode) {
        throw new Error(`Native convert does not support .${ext} yet`);
      } else {
        result = await convertToMarkdown(path);
      }
      if (cancelled || sourcePath !== path) return;
      markdown = result;
      setCached(path, result);
      status = "ready";
      queueMicrotask(() => mountEditorIfNeeded(result));
    } catch (e) {
      if (cancelled || sourcePath !== path) return;
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
    errorMessage = "";
    status = "idle";
    lastConvertedPath = null;
  }

  async function handleBrowseExternal() {
    try {
      const picked = await openDialog({
        multiple: false,
        filters: [
          {
            name: isNativeMode ? "Native-supported files" : "Convertible files",
            extensions: isNativeMode
              ? ["pdf", "html", "htm", "docx", "pptx"]
              : [
                  "pdf", "docx", "pptx", "xlsx", "xls",
                  "html", "htm", "epub", "ipynb",
                  "csv", "json", "xml",
                  "msg", "eml", "zip",
                ],
          },
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
          showSuccess(`Saved ${finalName}`);
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

  const isMissingUv = $derived(
    errorMessage.toLowerCase().includes("failed to spawn uvx"),
  );

  onMount(() => {
    if (!isWorkspaceMode && tab.path) {
      void runConvert(tab.path, false);
      lastConvertedPath = tab.path;
    }
  });

  onDestroy(() => {
    cancelled = true;
    if (slowTimer) clearTimeout(slowTimer);
    themeObserver?.disconnect();
    themeObserver = null;
    view?.destroy();
    view = null;
  });

  // Trigger convert when the workspace-mode user picks a source.
  $effect(() => {
    if (!isWorkspaceMode) return;
    const current = sourcePath;
    if (!current) return;
    if (current === lastConvertedPath) return;
    lastConvertedPath = current;
    void runConvert(current, false);
  });

  // Keep the tab title in sync with current source (workspace mode only).
  $effect(() => {
    if (!isWorkspaceMode) return;
    const prefix = isNativeMode ? "Native Convert" : "Convert";
    const title = sourcePath ? `${prefix}: ${basename(sourcePath)}` : prefix;
    if (tab.title !== title) workspace.patchTab(tab.id, { title });
  });

  // External reload token — re-run conversion with current source.
  $effect(() => {
    const token = tab.reloadToken ?? 0;
    if (token === 0 || !sourcePath) return;
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
        <span>{isNativeMode ? "Native Convert (playground)" : "Convert to Markdown"}</span>
      {/if}
    </div>
    <div class="flex items-center gap-2 shrink-0">
      {#if isWorkspaceMode && sourcePath}
        <Button size="xs" onclick={handleChangeSource}>
          <Icon name="arrow-left" size={14} class="mr-1" />
          Change
        </Button>
      {/if}
      {#if status === "ready"}
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
            {isNativeMode
              ? "Pick a PDF / HTML / DOCX / PPTX to test native conversion"
              : "Pick a file to convert to Markdown"}
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
            {#if isMissingUv && !isNativeMode}
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
        class:hidden={status !== "ready"}
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
