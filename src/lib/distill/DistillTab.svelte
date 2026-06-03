<script lang="ts">
  // Distill — the L0→L1 view of the multi-layer notes feature. A self-contained
  // full page: its OWN file explorer (left) + the extracted noun-ish terms
  // (right). It does NOT reuse the global Sidebar / FileTree / workspace store
  // and no longer follows a neighbouring pane — the source is whatever the user
  // picks in Distill's own explorer, uniformly across every transport.
  //
  // Read-only: no editor overlay, no DB write — a pure `path → annotations`
  // projection. Listing + extraction are transport-aware:
  //   - invoke → native IPC (real files via `list_workspace_files`).
  //   - http   → dev-only :7080 bridge (real files from a browser).
  //   - mock   → in-memory devmock (offline sample).
  import { untrack } from "svelte";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import {
    extractAnnotations,
    distillTransport,
    setDistillTransport,
    type Annotation,
    type DistillTransport,
  } from "$lib/workspace/tauri";
  import { basename } from "$lib/workspace/fileKind";
  import DistillExplorer from "./DistillExplorer.svelte";
  import DistillPanel from "./DistillPanel.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  // Normally self-contained (no props): Distill derives its source from its own
  // explorer and persists root/source to localStorage. The dev-only /gallery
  // can inject a transport + initial source to render the page OFFLINE without
  // reading or writing the user's real Distill localStorage — "preview mode".
  // In normal app use no props are passed, so behaviour is unchanged.
  let {
    transport: transportProp = undefined,
    initialRoot = undefined,
    initialSource = undefined,
  }: {
    transport?: DistillTransport;
    initialRoot?: string;
    initialSource?: string;
  } = $props();

  // Preview-injection props are read ONCE at init by design (the gallery passes
  // static values); untrack makes that intent explicit and avoids the
  // state_referenced_locally lint.
  const preview = untrack(() => transportProp !== undefined);

  // ── Transport (dev-only chip) ────────────────────────────────────────────
  // The :7080 HTTP bridge only exists in debug builds, so the chip is gated on
  // import.meta.env.DEV. The explorer re-lists automatically when `transport`
  // changes (it's a prop), and extraction re-runs via the $effect below.
  // Hidden in preview mode: cycling would write localStorage and switch to a
  // backend (http/invoke) that isn't reachable from the gallery.
  const showTransport = import.meta.env.DEV && !preview;
  const TRANSPORT_CYCLE: DistillTransport[] = ["invoke", "http", "mock"];
  const TRANSPORT_LABEL: Record<DistillTransport, string> = {
    invoke: "IPC",
    http: "HTTP",
    mock: "MOCK",
  };
  let transport = $state<DistillTransport>(untrack(() => transportProp ?? distillTransport()));

  // ── Own source state (root + picked file), persisted ─────────────────────
  const ROOT_KEY = "marrow.distill.root";
  const SOURCE_KEY = "marrow.distill.source";
  function lsGet(key: string): string {
    if (typeof localStorage === "undefined") return "";
    try {
      return localStorage.getItem(key) ?? "";
    } catch {
      return "";
    }
  }
  function lsSet(key: string, value: string): void {
    if (typeof localStorage === "undefined") return;
    try {
      if (value) localStorage.setItem(key, value);
      else localStorage.removeItem(key);
    } catch {
      // ignore — private mode / quota
    }
  }

  // Default root = a persisted choice, else the currently open workspace as a
  // convenient starting point (fully overridable — it's Distill's own copy).
  let root = $state(
    untrack(() =>
      preview
        ? (initialRoot ?? workspace.info?.root ?? "")
        : lsGet(ROOT_KEY) || workspace.info?.root || "",
    ),
  );
  let selected = $state<string | null>(
    untrack(() => (preview ? (initialSource ?? null) : lsGet(SOURCE_KEY) || null)),
  );

  function onroot(next: string): void {
    if (next === root) return;
    root = next;
    if (!preview) lsSet(ROOT_KEY, next);
    // The previous pick belonged to the old folder — drop it so the result
    // panel doesn't show stale terms for a file outside the new listing.
    selected = null;
    if (!preview) lsSet(SOURCE_KEY, "");
  }

  function onpick(path: string): void {
    selected = path || null;
    if (!preview) lsSet(SOURCE_KEY, selected ?? "");
    // sourcePath ($derived) updates → the $effect below re-extracts.
  }

  // ── Source resolution + extraction ───────────────────────────────────────
  const sourcePath = $derived(selected);

  let status = $state<"empty" | "loading" | "ready" | "error">("empty");
  let annotations = $state<Annotation[]>([]);
  let errorMessage = $state("");
  // Monotonic request id — guards against an older extraction resolving after
  // a newer one (race when switching notes quickly).
  let reqSeq = 0;

  async function run(path: string | null): Promise<void> {
    if (!path) {
      status = "empty";
      annotations = [];
      return;
    }
    const id = ++reqSeq;
    status = "loading";
    try {
      const result = await extractAnnotations(path, transport);
      if (id !== reqSeq) return; // superseded
      annotations = result;
      status = "ready";
    } catch (e) {
      if (id !== reqSeq) return;
      errorMessage = e instanceof Error ? e.message : String(e);
      status = "error";
    }
  }

  // Re-extract whenever the source OR the transport changes. Reading
  // `transport` here registers it as a dependency so cycling the chip
  // re-extracts the same file through the newly selected backend.
  $effect(() => {
    void transport;
    void run(sourcePath);
  });

  // Cycle IPC → HTTP → MOCK; persist the override.
  function cycleTransport(): void {
    const i = TRANSPORT_CYCLE.indexOf(transport);
    transport = TRANSPORT_CYCLE[(i + 1) % TRANSPORT_CYCLE.length];
    setDistillTransport(transport);
  }

  const totalTerms = $derived(annotations.length);
</script>

<div class="distill">
  <DistillExplorer {root} {transport} {selected} {onpick} {onroot} />

  <div class="distill-main">
    <header class="distill-header">
      <div class="distill-title">
        <Icon name="flask-conical" size={15} />
        <span>Distill</span>
        {#if sourcePath}
          <span class="distill-source" title={sourcePath}>{basename(sourcePath)}</span>
        {/if}
      </div>
      <div class="distill-actions">
        {#if showTransport}
          <button
            type="button"
            class="distill-mode"
            title={`提煉來源：${TRANSPORT_LABEL[transport]}（dev 切換 IPC / HTTP / MOCK）`}
            aria-label={`Distill transport: ${TRANSPORT_LABEL[transport]}`}
            onclick={cycleTransport}
          >
            {TRANSPORT_LABEL[transport]}
          </button>
        {/if}
        {#if status === "ready"}
          <span class="mw-meta">{totalTerms} terms</span>
        {/if}
        <button
          type="button"
          class="distill-refresh"
          title="Re-extract"
          aria-label="Re-extract"
          disabled={!sourcePath || status === "loading"}
          onclick={() => run(sourcePath)}
        >
          <Icon name="rotate-ccw" size={14} />
        </button>
      </div>
    </header>

    <DistillPanel {status} {annotations} {errorMessage} />
  </div>
</div>

<style>
  .distill {
    display: flex;
    flex-direction: row;
    height: 100%;
    width: 100%;
    background: var(--color-base-100);
    color: var(--color-base-content);
  }
  .distill-main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
  .distill-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--mw-rule);
    flex-shrink: 0;
  }
  .distill-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--mw-font-chrome);
    font-weight: 500;
    color: var(--mw-ink-1);
    min-width: 0;
  }
  .distill-source {
    color: var(--mw-ink-3);
    font-size: var(--mw-font-chrome-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .distill-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .distill-mode {
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.06em;
    color: var(--mw-ink-2);
    padding: 2px 6px;
    border: 1px solid var(--mw-rule);
    border-radius: var(--mw-radius-sm);
    background: transparent;
    transition: color 80ms ease, background 80ms ease;
  }
  .distill-mode:hover {
    color: var(--color-base-content);
    background: color-mix(in oklch, var(--mw-accent) 8%, transparent);
  }
  .distill-refresh {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: var(--mw-radius-sm);
    color: var(--mw-ink-2);
  }
  .distill-refresh:hover:not(:disabled) {
    background: color-mix(in oklch, var(--mw-accent) 8%, transparent);
    color: var(--color-base-content);
  }
  .distill-refresh:disabled {
    opacity: 0.4;
  }
</style>
