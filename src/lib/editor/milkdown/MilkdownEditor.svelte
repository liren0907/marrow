<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor, editorViewCtx, rootCtx, defaultValueCtx } from "@milkdown/core";
  import { TextSelection } from "@milkdown/prose/state";
  import type { Heading } from "$lib/workspace/tabRegistry.svelte";
  import { commonmark } from "@milkdown/preset-commonmark";
  import { gfm } from "@milkdown/preset-gfm";
  import { listener, listenerCtx } from "@milkdown/plugin-listener";
  import { history } from "@milkdown/plugin-history";
  import { clipboard } from "@milkdown/plugin-clipboard";
  import { indent } from "@milkdown/plugin-indent";
  import { prism } from "@milkdown/plugin-prism";
  import { math } from "@milkdown/plugin-math";
  import { diagram } from "@milkdown/plugin-diagram";
  import { configurePrism } from "./prism";
  import { imagePastePlugin } from "./imagePaste";
  import { slash, configSlash } from "./slashCommand";
  import {
    wikiLinkPlugin,
    wikiLinkConfigCtx,
    configWikiLinkSuggest,
    convertTextToWikiLinks,
    type WikiLinkClickHandler,
  } from "./wikiLink";
  import type { WikiLinkSuggestion } from "./wikiLink/suggest";
  import {
    transclusionPlugin,
    transclusionConfigCtx,
    configTransclusionSuggest,
    convertTextToTransclusions,
    type TransclusionClickHandler,
    type TransclusionSuggestion,
  } from "./transclusion";

  export type PeekRequest =
    | { kind: "wikilink"; target: string; section: string | null }
    | { kind: "word"; word: string };

  let {
    initial,
    onChange,
    onWikiLinkClick = null,
    getWikiLinkSuggestions = () => [],
    isWikiLinkResolved = () => true,
    onTransclusionClick = null,
    getTransclusionSuggestions = () => [],
    onReady = null,
    onOutlineUpdate = null,
    onPeekRequest = null,
  }: {
    initial: string;
    onChange: (md: string) => void;
    onWikiLinkClick?: WikiLinkClickHandler | null;
    getWikiLinkSuggestions?: (query: string) => WikiLinkSuggestion[];
    isWikiLinkResolved?: (target: string) => boolean;
    onTransclusionClick?: TransclusionClickHandler | null;
    getTransclusionSuggestions?: (query: string) => TransclusionSuggestion[];
    onReady?:
      | ((api: {
          scrollToPos: (pos: number) => void;
          peekAtCursor: () => void;
        }) => void)
      | null;
    onOutlineUpdate?: ((headings: Heading[]) => void) | null;
    onPeekRequest?: ((request: PeekRequest) => void) | null;
  } = $props();

  let host: HTMLDivElement;
  let editor: Editor | null = null;
  let destroyed = false;
  let errorMsg = $state<string | null>(null);
  let suppressNextChange = false;
  let outlineTimer: ReturnType<typeof setTimeout> | null = null;

  function pushOutline(): void {
    if (!editor || !onOutlineUpdate) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const out: Heading[] = [];
      view.state.doc.descendants((node, pos) => {
        if (node.type.name === "heading") {
          const rawLevel = node.attrs.level;
          const level = (typeof rawLevel === "number" ? rawLevel : parseInt(String(rawLevel), 10)) as 1 | 2 | 3 | 4 | 5 | 6;
          out.push({ level, text: node.textContent, pos });
        }
      });
      onOutlineUpdate!(out);
    });
  }

  function scheduleOutlinePush(): void {
    if (outlineTimer) clearTimeout(outlineTimer);
    outlineTimer = setTimeout(() => {
      outlineTimer = null;
      pushOutline();
    }, 200);
  }

  function scrollToPos(pos: number): void {
    if (!editor) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const resolved = view.state.doc.resolve(
        Math.min(Math.max(pos, 0), view.state.doc.content.size),
      );
      const tr = view.state.tr.setSelection(TextSelection.near(resolved));
      view.dispatch(tr.scrollIntoView());
      view.focus();
    });
  }

  function peekAtCursor(): void {
    if (!editor || !onPeekRequest) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const from = view.state.selection.$from;

      // Check if cursor is adjacent to a wikiLink inline node.
      const candidates = [from.nodeBefore, from.nodeAfter];
      for (const node of candidates) {
        if (!node || node.type.name !== "wikiLink") continue;
        const targetRaw = String(node.attrs.target ?? "");
        const hashIdx = targetRaw.indexOf("#");
        const target = hashIdx >= 0 ? targetRaw.slice(0, hashIdx) : targetRaw;
        const section = hashIdx >= 0 ? targetRaw.slice(hashIdx + 1) : null;
        onPeekRequest!({ kind: "wikilink", target, section });
        return;
      }

      // Fallback: extract word at cursor within the parent text block.
      const parent = from.parent;
      const offset = from.parentOffset;
      const blockText = parent.textContent;
      if (!blockText) return;
      const isWordChar = (ch: string): boolean => /[\p{L}\p{N}_-]/u.test(ch);
      let start = offset;
      let end = offset;
      while (start > 0 && isWordChar(blockText.charAt(start - 1))) start--;
      while (end < blockText.length && isWordChar(blockText.charAt(end))) end++;
      const word = blockText.slice(start, end).trim();
      if (word.length > 0) onPeekRequest!({ kind: "word", word });
    });
  }

  onMount(() => {
    (async () => {
      try {
        const instance = await Editor.make()
          .config((ctx) => {
            ctx.set(rootCtx, host);
            ctx.set(defaultValueCtx, initial);
            ctx.get(listenerCtx).markdownUpdated((_ctx, md, prev) => {
              if (suppressNextChange) {
                suppressNextChange = false;
                return;
              }
              if (md !== prev) {
                onChange(md);
                scheduleOutlinePush();
              }
            });
            configSlash(ctx);
            configurePrism(ctx);
            ctx.set(wikiLinkConfigCtx.key, {
              onClick: onWikiLinkClick,
              isResolved: isWikiLinkResolved,
            });
            configWikiLinkSuggest(ctx, getWikiLinkSuggestions);
            ctx.set(transclusionConfigCtx.key, {
              onClick: onTransclusionClick,
            });
            configTransclusionSuggest(ctx, getTransclusionSuggestions);
          })
          .use(commonmark)
          .use(gfm)
          .use(math)
          .use(diagram)
          .use(listener)
          .use(history)
          .use(imagePastePlugin)
          .use(clipboard)
          .use(indent)
          .use(prism)
          .use(slash)
          .use(wikiLinkPlugin)
          .use(transclusionPlugin)
          .create();

        if (destroyed) {
          instance.destroy();
          return;
        }
        editor = instance;

        // Convert literal [[...]] and ![[...]] text in the loaded doc into
        // their respective nodes, suppressing the listener so we don't fire a
        // spurious dirty change. Transclusion runs FIRST so the wiki-link
        // load-pass doesn't see `![[...]]` as text anymore.
        instance.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          suppressNextChange = true;
          const a = convertTextToTransclusions(view);
          const b = convertTextToWikiLinks(view);
          if (!a && !b) suppressNextChange = false;
        });

        // Push initial outline + hand the scroll / peek API to the parent.
        pushOutline();
        onReady?.({ scrollToPos, peekAtCursor });
      } catch (e) {
        errorMsg = e instanceof Error ? e.message : String(e);
        console.error("[Milkdown] create failed", e);
      }
    })();
  });

  onDestroy(() => {
    destroyed = true;
    if (outlineTimer) {
      clearTimeout(outlineTimer);
      outlineTimer = null;
    }
    if (editor) {
      try {
        editor.destroy();
      } catch (e) {
        console.warn("[Milkdown] destroy failed", e);
      }
      editor = null;
    }
  });
</script>

<div bind:this={host} class="milkdown-host w-full h-full overflow-y-auto"></div>
{#if errorMsg}
  <div class="absolute top-2 right-2 text-xs text-error bg-base-100 border border-error px-2 py-1 rounded">
    Milkdown: {errorMsg}
  </div>
{/if}

<style>
  :global(.milkdown-host) {
    scrollbar-width: none;
  }
  :global(.milkdown-host::-webkit-scrollbar) {
    display: none;
  }
  :global(.milkdown-host .milkdown) {
    outline: none;
    min-height: 100%;
  }
  :global(.milkdown-host .ProseMirror) {
    outline: none;
    min-height: 100%;
    max-width: 46rem;
    margin: 0 auto;
    padding: 2.5rem 3rem 6rem 3rem;
    font-size: 15px;
    line-height: 1.7;
    color: var(--color-base-content);
  }
  :global(.milkdown-host .ProseMirror h1) {
    font-size: 1.875rem;
    font-weight: 700;
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    text-align: left;
    line-height: 1.25;
  }
  :global(.milkdown-host .ProseMirror h2) {
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 1.25rem;
    margin-bottom: 0.5rem;
    line-height: 1.3;
  }
  :global(.milkdown-host .ProseMirror h3) {
    font-size: 1.25rem;
    font-weight: 600;
    margin-top: 1rem;
    margin-bottom: 0.5rem;
  }
  :global(.milkdown-host .ProseMirror h4, .milkdown-host .ProseMirror h5, .milkdown-host .ProseMirror h6) {
    font-size: 1rem;
    font-weight: 600;
    margin-top: 0.75rem;
    margin-bottom: 0.25rem;
  }
  :global(.milkdown-host .ProseMirror p) {
    margin-bottom: 0.5rem;
  }
  :global(.milkdown-host .ProseMirror ul),
  :global(.milkdown-host .ProseMirror ol) {
    padding-left: 1.5rem;
    margin-bottom: 0.75rem;
  }
  :global(.milkdown-host .ProseMirror ul) {
    list-style: disc;
  }
  :global(.milkdown-host .ProseMirror ol) {
    list-style: decimal;
  }
  :global(.milkdown-host .ProseMirror li) {
    margin-bottom: 0.25rem;
  }
  :global(.milkdown-host .ProseMirror li > p) {
    margin-bottom: 0.25rem;
  }
  :global(.milkdown-host .ProseMirror code) {
    font-family: var(--font-mono);
    background-color: var(--color-base-200);
    padding: 0.125rem 0.35rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
  }
  :global(.milkdown-host .ProseMirror pre) {
    font-family: var(--font-mono);
    background-color: var(--color-base-200);
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    margin-bottom: 0.75rem;
    font-size: 0.875em;
  }
  :global(.milkdown-host .ProseMirror pre code) {
    background: none;
    padding: 0;
  }
  :global(.milkdown-host .ProseMirror blockquote) {
    border-left: 3px solid color-mix(in oklch, var(--color-primary) 40%, transparent);
    padding-left: 1rem;
    color: color-mix(in oklch, var(--color-base-content) 70%, transparent);
    margin: 0.75rem 0;
  }
  :global(.milkdown-host .ProseMirror a) {
    color: var(--color-primary);
    text-decoration: underline;
  }
  :global(.milkdown-host .ProseMirror table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 0.75rem;
  }
  :global(.milkdown-host .ProseMirror th),
  :global(.milkdown-host .ProseMirror td) {
    border: 1px solid var(--color-base-300);
    padding: 0.5rem 0.75rem;
    text-align: left;
  }
  :global(.milkdown-host .ProseMirror th) {
    background-color: var(--color-base-200);
    font-weight: 600;
  }
  :global(.milkdown-host .ProseMirror hr) {
    border: none;
    border-top: 1px solid var(--color-base-300);
    margin: 1.5rem 0;
  }

  :global(.marrow-slash-menu) {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 50;
    min-width: 14rem;
    max-width: 20rem;
    padding: 0.25rem;
    background-color: var(--color-base-100);
    border: 1px solid var(--color-base-300);
    border-radius: 0.5rem;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.18);
    display: none;
    flex-direction: column;
    gap: 0.125rem;
  }
  :global(.marrow-slash-menu[data-show="true"]) {
    display: flex;
  }
  :global(.marrow-slash-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.4rem 0.6rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: var(--color-base-content);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  :global(.marrow-slash-item.selected),
  :global(.marrow-slash-item:hover) {
    background-color: var(--color-base-200);
  }
  :global(.marrow-slash-item .slash-hint) {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.75rem;
    color: color-mix(in oklch, var(--color-base-content) 50%, transparent);
  }

  :global(.milkdown-host .ProseMirror .wiki-link) {
    display: inline;
    color: var(--color-primary);
    background-color: color-mix(in oklch, var(--color-primary) 8%, transparent);
    padding: 0.05rem 0.3rem;
    border-radius: 0.25rem;
    cursor: pointer;
    text-decoration: none;
    font-weight: 500;
  }
  :global(.milkdown-host .ProseMirror .wiki-link:hover) {
    background-color: color-mix(in oklch, var(--color-primary) 15%, transparent);
  }
  :global(.milkdown-host .ProseMirror .wiki-link.unresolved) {
    color: color-mix(in oklch, var(--color-base-content) 40%, transparent);
    background-color: transparent;
    border-bottom: 1px dashed color-mix(in oklch, var(--color-base-content) 30%, transparent);
    border-radius: 0;
    padding: 0;
  }
  :global(.milkdown-host .ProseMirror .wiki-link.unresolved:hover) {
    background-color: color-mix(in oklch, var(--color-base-content) 5%, transparent);
  }

  :global(.marrow-wikilink-menu) {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 50;
    min-width: 16rem;
    max-width: 24rem;
    padding: 0.25rem;
    background-color: var(--color-base-100);
    border: 1px solid var(--color-base-300);
    border-radius: 0.5rem;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.18);
    display: none;
    flex-direction: column;
    gap: 0.125rem;
  }
  :global(.marrow-wikilink-menu[data-show="true"]) {
    display: flex;
  }
  :global(.marrow-wikilink-empty) {
    padding: 0.5rem 0.6rem;
    font-size: 0.8125rem;
    color: color-mix(in oklch, var(--color-base-content) 50%, transparent);
    font-style: italic;
  }
  :global(.marrow-wikilink-item) {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.4rem 0.6rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: var(--color-base-content);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  :global(.marrow-wikilink-item.selected),
  :global(.marrow-wikilink-item:hover) {
    background-color: var(--color-base-200);
  }
  :global(.marrow-wikilink-item .wl-folder) {
    font-size: 0.75rem;
    color: color-mix(in oklch, var(--color-base-content) 50%, transparent);
  }

  /* Prism token colors — light variant (default). Tuned against github-light. */
  :global(.milkdown-host .ProseMirror .token.comment),
  :global(.milkdown-host .ProseMirror .token.prolog),
  :global(.milkdown-host .ProseMirror .token.doctype),
  :global(.milkdown-host .ProseMirror .token.cdata) {
    color: #6a737d;
    font-style: italic;
  }
  :global(.milkdown-host .ProseMirror .token.punctuation) {
    color: #24292e;
  }
  :global(.milkdown-host .ProseMirror .token.property),
  :global(.milkdown-host .ProseMirror .token.tag),
  :global(.milkdown-host .ProseMirror .token.boolean),
  :global(.milkdown-host .ProseMirror .token.number),
  :global(.milkdown-host .ProseMirror .token.constant),
  :global(.milkdown-host .ProseMirror .token.symbol),
  :global(.milkdown-host .ProseMirror .token.deleted) {
    color: #005cc5;
  }
  :global(.milkdown-host .ProseMirror .token.selector),
  :global(.milkdown-host .ProseMirror .token.attr-name),
  :global(.milkdown-host .ProseMirror .token.string),
  :global(.milkdown-host .ProseMirror .token.char),
  :global(.milkdown-host .ProseMirror .token.builtin),
  :global(.milkdown-host .ProseMirror .token.inserted) {
    color: #032f62;
  }
  :global(.milkdown-host .ProseMirror .token.operator),
  :global(.milkdown-host .ProseMirror .token.entity),
  :global(.milkdown-host .ProseMirror .token.url),
  :global(.milkdown-host .ProseMirror .language-css .token.string),
  :global(.milkdown-host .ProseMirror .style .token.string) {
    color: #d73a49;
  }
  :global(.milkdown-host .ProseMirror .token.atrule),
  :global(.milkdown-host .ProseMirror .token.attr-value),
  :global(.milkdown-host .ProseMirror .token.keyword) {
    color: #d73a49;
  }
  :global(.milkdown-host .ProseMirror .token.function),
  :global(.milkdown-host .ProseMirror .token.class-name) {
    color: #6f42c1;
  }
  :global(.milkdown-host .ProseMirror .token.regex),
  :global(.milkdown-host .ProseMirror .token.important),
  :global(.milkdown-host .ProseMirror .token.variable) {
    color: #e36209;
  }
  :global(.milkdown-host .ProseMirror .token.important),
  :global(.milkdown-host .ProseMirror .token.bold) {
    font-weight: bold;
  }
  :global(.milkdown-host .ProseMirror .token.italic) {
    font-style: italic;
  }

  /* Dark variant — tuned against github-dark. */
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.comment),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.prolog),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.doctype),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.cdata) {
    color: #8b949e;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.punctuation) {
    color: #c9d1d9;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.property),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.tag),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.boolean),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.number),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.constant),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.symbol),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.deleted) {
    color: #79c0ff;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.selector),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.attr-name),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.string),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.char),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.builtin),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.inserted) {
    color: #a5d6ff;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.operator),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.entity),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.url),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.atrule),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.attr-value),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.keyword) {
    color: #ff7b72;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.function),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.class-name) {
    color: #d2a8ff;
  }
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.regex),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.important),
  :global([data-theme="dark"] .milkdown-host, [data-theme="marrow-pro-dark"] .milkdown-host .ProseMirror .token.variable) {
    color: #ffa657;
  }

  /* Transclusion embed */
  :global(.milkdown-host .ProseMirror .transclusion-embed) {
    border: 1px solid var(--color-base-300);
    border-radius: 0.5rem;
    margin: 1rem 0;
    background-color: var(--color-base-100);
    overflow: hidden;
  }
  :global(.milkdown-host .ProseMirror .transclusion-embed.unresolved) {
    border-style: dashed;
    border-color: color-mix(in oklch, var(--color-base-content) 25%, transparent);
  }
  :global(.milkdown-host .ProseMirror .transclusion-header) {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.75rem;
    background-color: var(--color-base-200);
    border-bottom: 1px solid var(--color-base-300);
    cursor: pointer;
    font-size: 0.8125rem;
    color: color-mix(in oklch, var(--color-base-content) 75%, transparent);
    user-select: none;
  }
  :global(.milkdown-host .ProseMirror .transclusion-header:hover) {
    background-color: var(--color-base-300);
    color: var(--color-base-content);
  }
  :global(.milkdown-host .ProseMirror .transclusion-header .transclusion-target) {
    font-weight: 500;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content) {
    padding: 0.75rem 1rem;
    font-size: 0.9em;
    line-height: 1.6;
    color: color-mix(in oklch, var(--color-base-content) 85%, transparent);
    max-height: 480px;
    overflow-y: auto;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content h1),
  :global(.milkdown-host .ProseMirror .transclusion-content h2),
  :global(.milkdown-host .ProseMirror .transclusion-content h3) {
    font-size: 1em;
    font-weight: 600;
    margin: 0.5em 0 0.25em;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content p) {
    margin: 0.4em 0;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content ul),
  :global(.milkdown-host .ProseMirror .transclusion-content ol) {
    padding-left: 1.5rem;
    margin: 0.4em 0;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content code) {
    background-color: var(--color-base-200);
    padding: 0.05rem 0.3rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content pre) {
    background-color: var(--color-base-200);
    padding: 0.75rem;
    border-radius: 0.375rem;
    overflow-x: auto;
    font-size: 0.875em;
  }
  :global(.milkdown-host .ProseMirror .transclusion-content .transclusion-meta) {
    color: color-mix(in oklch, var(--color-base-content) 40%, transparent);
    font-style: italic;
  }
  :global(.milkdown-host .ProseMirror .transclusion-image) {
    display: block;
    max-width: 100%;
    max-height: 480px;
    margin: 0 auto;
    object-fit: contain;
  }
  :global(.milkdown-host .ProseMirror .transclusion-video) {
    display: block;
    max-width: 100%;
    max-height: 480px;
    margin: 0 auto;
  }
  :global(.milkdown-host .ProseMirror .transclusion-audio) {
    display: block;
    width: 100%;
  }
</style>
