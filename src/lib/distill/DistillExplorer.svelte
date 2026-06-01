<script lang="ts">
  // Distill's OWN file explorer — a self-contained left column. It does NOT
  // use the global Sidebar / FileTree / workspace store: it lists the `.md`
  // files under its own `root` via `listDistillSources` (transport-aware) and
  // reports selections back to DistillTab. Flat, searchable list (no nested
  // tree yet — by design).
  import { onMount } from "svelte";
  import { isInTauri } from "$lib/devmock/detect";
  import { openDirectoryDialog, type DistillTransport } from "$lib/workspace/tauri";
  import { listDistillSources, type DistillSource } from "./distillSource";
  import Icon from "$lib/components/ui/Icon.svelte";

  let {
    root,
    transport,
    selected,
    onpick,
    onroot,
  }: {
    root: string;
    transport: DistillTransport;
    selected: string | null;
    onpick: (path: string) => void;
    onroot: (root: string) => void;
  } = $props();

  let entries = $state<DistillSource[]>([]);
  let status = $state<"idle" | "loading" | "ready" | "error">("idle");
  let error = $state("");
  let filter = $state("");
  // Editable mirror of `root` so typing doesn't commit on every keystroke.
  // Initialized empty and populated by the effect below (referencing the prop
  // directly in the initializer would only capture its first value).
  let rootDraft = $state("");

  // Keep the input in sync with `root` from the outside (the persisted default
  // on mount, or the native folder dialog) without clobbering mid-edit — root
  // only changes on an explicit commit/browse, never on keystroke.
  $effect(() => {
    rootDraft = root;
  });

  // (Re)list whenever the root or transport changes.
  $effect(() => {
    const r = root.trim();
    const t = transport;
    if (!r) {
      entries = [];
      status = "idle";
      return;
    }
    let cancelled = false;
    status = "loading";
    error = "";
    listDistillSources(r, t)
      .then((res) => {
        if (cancelled) return;
        entries = res;
        status = "ready";
      })
      .catch((e) => {
        if (cancelled) return;
        error = e instanceof Error ? e.message : String(e);
        status = "error";
      });
    return () => {
      cancelled = true;
    };
  });

  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return entries;
    return entries.filter((e) => e.rel.toLowerCase().includes(q));
  });

  function commitRoot(value: string): void {
    onroot(value.trim());
  }

  async function browse(): Promise<void> {
    try {
      const picked = await openDirectoryDialog();
      if (picked) onroot(picked);
    } catch {
      // user cancelled / dialog unavailable — leave root as-is
    }
  }
</script>

<aside class="dx">
  <div class="dx-root">
    <input
      class="dx-root-input"
      type="text"
      placeholder="資料夾路徑…"
      bind:value={rootDraft}
      spellcheck="false"
      title={root}
      onchange={(e) => commitRoot(e.currentTarget.value)}
      onkeydown={(e) => {
        if (e.key === "Enter") commitRoot(e.currentTarget.value);
      }}
    />
    {#if isInTauri}
      <button
        type="button"
        class="dx-browse"
        title="選擇資料夾…"
        aria-label="選擇資料夾"
        onclick={browse}
      >
        <Icon name="folder-open" size={14} />
      </button>
    {/if}
  </div>

  {#if status === "ready" && entries.length > 0}
    <div class="dx-filter">
      <Icon name="search" size={12} class="dx-filter-icon" />
      <input
        class="dx-filter-input"
        type="text"
        placeholder="過濾…"
        bind:value={filter}
        spellcheck="false"
      />
    </div>
  {/if}

  <div class="dx-list">
    {#if status === "idle"}
      <p class="dx-hint">選一個資料夾來列出 <code>.md</code>。</p>
    {:else if status === "loading"}
      <p class="dx-hint">載入中…</p>
    {:else if status === "error"}
      <p class="dx-hint dx-error" title={error}>讀取失敗：{error}</p>
    {:else if entries.length === 0}
      <p class="dx-hint">此資料夾沒有 <code>.md</code>。</p>
    {:else if filtered.length === 0}
      <p class="dx-hint">沒有符合「{filter}」的檔案。</p>
    {:else}
      <ul class="dx-files">
        {#each filtered as entry (entry.path)}
          <li>
            <button
              type="button"
              class="dx-file"
              class:active={entry.path === selected}
              title={entry.path}
              onclick={() => onpick(entry.path)}
            >
              <Icon name="file-text" size={13} class="dx-file-icon" />
              <span class="dx-file-rel">{entry.rel}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  {#if status === "ready" && entries.length > 0}
    <div class="dx-footer mw-meta">
      {filtered.length === entries.length
        ? `${entries.length} files`
        : `${filtered.length} / ${entries.length}`}
    </div>
  {/if}
</aside>

<style>
  .dx {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 240px;
    flex-shrink: 0;
    border-right: 1px solid var(--mw-rule);
    background: var(--color-base-200);
    min-width: 0;
  }
  .dx-root {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--mw-rule);
    flex-shrink: 0;
  }
  .dx-root-input {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-base-content);
    background: var(--color-base-100);
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    padding: 4px 6px;
  }
  .dx-root-input:focus {
    outline: none;
    border-color: var(--mw-accent);
  }
  .dx-browse {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    flex-shrink: 0;
    border-radius: var(--mw-radius-sm);
    color: var(--mw-ink-2);
  }
  .dx-browse:hover {
    background: color-mix(in oklch, var(--mw-accent) 8%, transparent);
    color: var(--color-base-content);
  }
  .dx-filter {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--mw-rule);
    flex-shrink: 0;
    color: var(--mw-ink-3);
  }
  .dx-filter-input {
    flex: 1;
    min-width: 0;
    font-size: 11px;
    color: var(--color-base-content);
    background: transparent;
    border: none;
  }
  .dx-filter-input:focus {
    outline: none;
  }
  .dx-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
    min-height: 0;
  }
  .dx-hint {
    color: var(--mw-ink-3);
    font-size: var(--text-xs);
    padding: 10px 8px;
    line-height: 1.5;
  }
  .dx-error {
    color: var(--color-error);
    word-break: break-word;
  }
  .dx-files {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .dx-file {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    text-align: left;
    padding: 4px 6px;
    border-radius: var(--mw-radius-xs);
    color: var(--mw-ink-1);
    min-width: 0;
  }
  .dx-file:hover {
    background: color-mix(in oklch, var(--mw-accent) 6%, transparent);
  }
  .dx-file.active {
    background: color-mix(in oklch, var(--mw-accent) 14%, transparent);
    color: var(--color-base-content);
  }
  .dx-file-rel {
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dx-footer {
    padding: 4px 8px;
    border-top: 1px solid var(--mw-rule);
    flex-shrink: 0;
  }
</style>
