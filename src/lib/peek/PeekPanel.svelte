<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { peek } from "./peekState.svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    renderEmbedded,
    renderEmbeddedSection,
  } from "$lib/editor/milkdown/transclusion/renderer";
  import { debounce } from "$lib/utils/debounce";

  let container: HTMLDivElement;
  // contentHost is re-bound each render via bind:this and the effect mutates
  // its innerHTML based on peek state — must be reactive so the effect re-runs
  // when mount/unmount happens. It's a plain HTMLDivElement, not an editor
  // handle (Rule #1 doesn't apply).
  let contentHost = $state<HTMLDivElement | null>(null);

  const current = $derived(peek.current);

  const html = $derived.by(() => {
    if (!current) return "";
    return current.section
      ? renderEmbeddedSection(current.content, current.section)
      : renderEmbedded(current.content);
  });

  $effect(() => {
    if (!contentHost) return;
    contentHost.innerHTML = html;
    enhanceWikiLinks(contentHost);
    // Restore remembered scroll for this layer on next frame.
    const target = current;
    if (!target) return;
    queueMicrotask(() => {
      if (contentHost) contentHost.scrollTop = target.scrollY;
    });
  });

  function enhanceWikiLinks(host: HTMLElement): void {
    const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT, {
      acceptNode(n: Node) {
        const parent = (n as Text).parentElement;
        if (!parent) return NodeFilter.FILTER_REJECT;
        if (parent.closest("pre, code, a"))
          return NodeFilter.FILTER_REJECT;
        return /\[\[[^\]]+\]\]/.test((n as Text).nodeValue ?? "")
          ? NodeFilter.FILTER_ACCEPT
          : NodeFilter.FILTER_SKIP;
      },
    });
    const matches: Text[] = [];
    let node = walker.nextNode();
    while (node) {
      matches.push(node as Text);
      node = walker.nextNode();
    }
    for (const t of matches) replaceWikiLinksInTextNode(t);
  }

  function replaceWikiLinksInTextNode(t: Text): void {
    const text = t.nodeValue ?? "";
    const frag = document.createDocumentFragment();
    let last = 0;
    const re = /\[\[([^[\]\n|]+)(?:\|([^[\]\n]+))?\]\]/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(text)) !== null) {
      if (m.index > last)
        frag.append(document.createTextNode(text.slice(last, m.index)));
      const targetRaw = m[1];
      const display = m[2];
      const hashIdx = targetRaw.indexOf("#");
      const target = hashIdx >= 0 ? targetRaw.slice(0, hashIdx) : targetRaw;
      const section = hashIdx >= 0 ? targetRaw.slice(hashIdx + 1) : "";
      const span = document.createElement("span");
      span.className = "peek-wikilink";
      span.dataset.target = target;
      if (section) span.dataset.section = section;
      span.textContent = display ?? targetRaw;
      frag.append(span);
      last = m.index + m[0].length;
    }
    if (last < text.length)
      frag.append(document.createTextNode(text.slice(last)));
    t.parentNode?.replaceChild(frag, t);
  }

  function handleClick(e: MouseEvent): void {
    const el = (e.target as HTMLElement | null)?.closest(
      ".peek-wikilink",
    ) as HTMLElement | null;
    if (!el) return;
    e.preventDefault();
    const t = el.dataset.target;
    if (!t) return;
    const section = el.dataset.section ?? null;
    const path = workspace.resolveBasename(t);
    if (!path) return;
    void peek.push(path, section);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      if (e.metaKey || e.ctrlKey) {
        peek.clear();
      } else {
        peek.pop();
      }
    }
  }

  const rememberScroll = debounce(() => {
    if (contentHost) peek.rememberScroll(contentHost.scrollTop);
  }, 150);

  function handleScroll(): void {
    rememberScroll();
  }

  function goToLayer(targetDepth: number): void {
    peek.popTo(targetDepth);
  }

  // Auto-focus the container so Esc / Cmd+Esc work immediately after push.
  $effect(() => {
    void peek.depth;
    if (container && peek.isOpen) {
      queueMicrotask(() => container?.focus());
    }
  });

  onMount(() => {
    if (container) container.focus();
  });

  onDestroy(() => {
    // State lives in a module — don't clear on unmount. Only explicit
    // clear / pop should empty the stack.
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={container}
  class="flex flex-col flex-1 min-h-0 outline-none"
  tabindex="0"
  onkeydown={handleKeydown}
  role="region"
  aria-label="Peek panel"
>
  {#if peek.isOpen}
    <div
      class="flex items-center gap-1 px-3 py-1.5 border-b border-base-200 shrink-0 overflow-x-auto text-[11px]"
    >
      {#each peek.layers as layer, i (i + ":" + layer.path + ":" + (layer.section ?? ""))}
        {#if i > 0}
          <span class="text-base-content/30">›</span>
        {/if}
        <button
          type="button"
          class="truncate max-w-[220px] hover:bg-base-200 px-1.5 py-0.5 rounded"
          class:font-semibold={i === peek.layers.length - 1}
          class:text-primary={i === peek.layers.length - 1}
          onclick={() => goToLayer(i + 1)}
          title={layer.path}
        >
          {layer.label}
        </button>
      {/each}
      <div class="flex-1"></div>
      <span class="text-[10px] text-base-content/40">
        {peek.depth}/5
      </span>
      <button
        type="button"
        class="btn btn-ghost btn-xs btn-square ml-1"
        title="Clear peek stack"
        onclick={() => peek.clear()}
        aria-label="Clear peek stack"
      >
        <span class="material-symbols-rounded text-[14px]">close</span>
      </button>
    </div>

    <div
      bind:this={contentHost}
      class="peek-content flex-1 min-h-0 overflow-y-auto px-4 py-3 text-sm leading-relaxed prose prose-sm max-w-none"
      onscroll={handleScroll}
    ></div>
  {:else}
    <div
      class="flex-1 flex items-center justify-center text-xs text-base-content/40 italic"
    >
      No peek active — place cursor on a <code class="mx-1">[[link]]</code> and
      press ⌘⇧Space
    </div>
  {/if}
</div>

<style>
  :global(.peek-content .peek-wikilink) {
    color: oklch(var(--p));
    cursor: pointer;
    text-decoration: underline;
    text-decoration-style: dotted;
    text-underline-offset: 2px;
  }
  :global(.peek-content .peek-wikilink:hover) {
    background-color: oklch(var(--p) / 0.1);
    border-radius: 3px;
  }
  :global(.peek-content h1),
  :global(.peek-content h2),
  :global(.peek-content h3),
  :global(.peek-content h4) {
    font-weight: 600;
    margin-top: 0.75rem;
    margin-bottom: 0.25rem;
  }
  :global(.peek-content h1) {
    font-size: 1.1rem;
  }
  :global(.peek-content h2) {
    font-size: 1rem;
  }
  :global(.peek-content h3) {
    font-size: 0.9rem;
  }
  :global(.peek-content p) {
    margin: 0.5rem 0;
  }
  :global(.peek-content pre) {
    font-size: 0.75rem;
    background: oklch(var(--b2));
    padding: 0.5rem;
    border-radius: 4px;
    overflow-x: auto;
  }
  :global(.peek-content code) {
    font-size: 0.85em;
    background: oklch(var(--b2));
    padding: 0.1em 0.3em;
    border-radius: 3px;
  }
  :global(.peek-content pre code) {
    background: transparent;
    padding: 0;
  }
  :global(.peek-content ul),
  :global(.peek-content ol) {
    padding-left: 1.25rem;
    margin: 0.5rem 0;
  }
  :global(.peek-content blockquote) {
    border-left: 3px solid oklch(var(--bc) / 0.2);
    padding-left: 0.75rem;
    color: oklch(var(--bc) / 0.7);
    margin: 0.5rem 0;
  }
</style>
