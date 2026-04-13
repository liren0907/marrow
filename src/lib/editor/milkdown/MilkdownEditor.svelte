<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor, editorViewCtx, rootCtx, defaultValueCtx } from "@milkdown/core";
  import { commonmark } from "@milkdown/preset-commonmark";
  import { gfm } from "@milkdown/preset-gfm";
  import { listener, listenerCtx } from "@milkdown/plugin-listener";
  import { history } from "@milkdown/plugin-history";
  import { clipboard } from "@milkdown/plugin-clipboard";
  import { indent } from "@milkdown/plugin-indent";
  import { prism } from "@milkdown/plugin-prism";
  import { configurePrism } from "./prism";
  import { slash, configSlash } from "./slashCommand";
  import {
    wikiLinkPlugin,
    wikiLinkConfigCtx,
    configWikiLinkSuggest,
    convertTextToWikiLinks,
    type WikiLinkClickHandler,
  } from "./wikiLink";
  import type { WikiLinkSuggestion } from "./wikiLink/suggest";

  let {
    initial,
    onChange,
    onWikiLinkClick = null,
    getWikiLinkSuggestions = () => [],
    isWikiLinkResolved = () => true,
  }: {
    initial: string;
    onChange: (md: string) => void;
    onWikiLinkClick?: WikiLinkClickHandler | null;
    getWikiLinkSuggestions?: (query: string) => WikiLinkSuggestion[];
    isWikiLinkResolved?: (target: string) => boolean;
  } = $props();

  let host: HTMLDivElement;
  let editor: Editor | null = null;
  let destroyed = false;
  let errorMsg = $state<string | null>(null);
  let suppressNextChange = false;

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
              if (md !== prev) onChange(md);
            });
            configSlash(ctx);
            configurePrism(ctx);
            ctx.set(wikiLinkConfigCtx.key, {
              onClick: onWikiLinkClick,
              isResolved: isWikiLinkResolved,
            });
            configWikiLinkSuggest(ctx, getWikiLinkSuggestions);
          })
          .use(commonmark)
          .use(gfm)
          .use(listener)
          .use(history)
          .use(clipboard)
          .use(indent)
          .use(prism)
          .use(slash)
          .use(wikiLinkPlugin)
          .create();

        if (destroyed) {
          instance.destroy();
          return;
        }
        editor = instance;

        // Convert literal [[...]] text in the loaded doc into wikiLink nodes,
        // suppressing the listener so we don't fire a spurious dirty change.
        instance.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          suppressNextChange = true;
          const changed = convertTextToWikiLinks(view);
          if (!changed) suppressNextChange = false;
        });
      } catch (e) {
        errorMsg = e instanceof Error ? e.message : String(e);
        console.error("[Milkdown] create failed", e);
      }
    })();
  });

  onDestroy(() => {
    destroyed = true;
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
    color: oklch(var(--bc));
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
    margin-bottom: 0.75rem;
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
    background-color: oklch(var(--b2));
    padding: 0.125rem 0.35rem;
    border-radius: 0.25rem;
    font-size: 0.875em;
  }
  :global(.milkdown-host .ProseMirror pre) {
    font-family: var(--font-mono);
    background-color: oklch(var(--b2));
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
    border-left: 3px solid oklch(var(--p) / 0.4);
    padding-left: 1rem;
    color: oklch(var(--bc) / 0.7);
    margin: 0.75rem 0;
  }
  :global(.milkdown-host .ProseMirror a) {
    color: oklch(var(--p));
    text-decoration: underline;
  }
  :global(.milkdown-host .ProseMirror table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 0.75rem;
  }
  :global(.milkdown-host .ProseMirror th),
  :global(.milkdown-host .ProseMirror td) {
    border: 1px solid oklch(var(--b3));
    padding: 0.5rem 0.75rem;
    text-align: left;
  }
  :global(.milkdown-host .ProseMirror th) {
    background-color: oklch(var(--b2));
    font-weight: 600;
  }
  :global(.milkdown-host .ProseMirror hr) {
    border: none;
    border-top: 1px solid oklch(var(--b3));
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
    background-color: oklch(var(--b1));
    border: 1px solid oklch(var(--b3));
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
    color: oklch(var(--bc));
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  :global(.marrow-slash-item.selected),
  :global(.marrow-slash-item:hover) {
    background-color: oklch(var(--b2));
  }
  :global(.marrow-slash-item .slash-hint) {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 0.75rem;
    color: oklch(var(--bc) / 0.5);
  }

  :global(.milkdown-host .ProseMirror .wiki-link) {
    display: inline;
    color: oklch(var(--p));
    background-color: oklch(var(--p) / 0.08);
    padding: 0.05rem 0.3rem;
    border-radius: 0.25rem;
    cursor: pointer;
    text-decoration: none;
    font-weight: 500;
  }
  :global(.milkdown-host .ProseMirror .wiki-link:hover) {
    background-color: oklch(var(--p) / 0.15);
  }
  :global(.milkdown-host .ProseMirror .wiki-link.unresolved) {
    color: oklch(var(--bc) / 0.4);
    background-color: transparent;
    border-bottom: 1px dashed oklch(var(--bc) / 0.3);
    border-radius: 0;
    padding: 0;
  }
  :global(.milkdown-host .ProseMirror .wiki-link.unresolved:hover) {
    background-color: oklch(var(--bc) / 0.05);
  }

  :global(.marrow-wikilink-menu) {
    position: absolute;
    top: 0;
    left: 0;
    z-index: 50;
    min-width: 16rem;
    max-width: 24rem;
    padding: 0.25rem;
    background-color: oklch(var(--b1));
    border: 1px solid oklch(var(--b3));
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
    color: oklch(var(--bc) / 0.5);
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
    color: oklch(var(--bc));
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
  }
  :global(.marrow-wikilink-item.selected),
  :global(.marrow-wikilink-item:hover) {
    background-color: oklch(var(--b2));
  }
  :global(.marrow-wikilink-item .wl-folder) {
    font-size: 0.75rem;
    color: oklch(var(--bc) / 0.5);
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
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.comment),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.prolog),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.doctype),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.cdata) {
    color: #8b949e;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.punctuation) {
    color: #c9d1d9;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.property),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.tag),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.boolean),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.number),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.constant),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.symbol),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.deleted) {
    color: #79c0ff;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.selector),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.attr-name),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.string),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.char),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.builtin),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.inserted) {
    color: #a5d6ff;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.operator),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.entity),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.url),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.atrule),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.attr-value),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.keyword) {
    color: #ff7b72;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.function),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.class-name) {
    color: #d2a8ff;
  }
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.regex),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.important),
  :global([data-theme="dark"] .milkdown-host .ProseMirror .token.variable) {
    color: #ffa657;
  }
</style>
