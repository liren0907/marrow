<script lang="ts">
  // Distill — the L0→L1 view of the multi-layer notes feature. A panel-only tab:
  // it renders ONLY the extracted noun-ish terms (DistillPanel) and follows the
  // markdown open in a NEIGHBOURING pane. There's no embedded editor and no own
  // explorer — the L0 source is just a normal MarkdownTab sitting beside it, so
  // you edit your note as usual and the panel re-extracts when you switch notes
  // or save. Extraction is transport-aware:
  //   - invoke → native IPC (reads the real file by path).
  //   - http   → dev-only :7080 bridge (real files from a browser).
  //   - mock   → in-memory devmock (offline sample / gallery preview).
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
  import DistillPanel from "./DistillPanel.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";

  // Normally no props: Distill follows the neighbouring markdown pane. The
  // dev-only /gallery injects a transport + a fixed source path to render the
  // tab OFFLINE without a real pane behind it — "preview mode". In normal app
  // use no props are passed, so behaviour is unchanged.
  let {
    transport: transportProp = undefined,
    previewSource = undefined,
  }: {
    transport?: DistillTransport;
    previewSource?: string;
  } = $props();

  // Read ONCE at init by design (the gallery passes static values); untrack
  // makes that intent explicit and avoids the state_referenced_locally lint.
  const preview = untrack(() => transportProp !== undefined);

  // ── Transport (dev-only chip) ────────────────────────────────────────────
  // The :7080 HTTP bridge only exists in debug builds, so the chip is gated on
  // import.meta.env.DEV. Extraction re-runs via the $effect below when the
  // transport changes. Hidden in preview mode: cycling would switch to a backend
  // (http/invoke) that isn't reachable from the gallery.
  const showTransport = import.meta.env.DEV && !preview;
  const TRANSPORT_CYCLE: DistillTransport[] = ["invoke", "http", "mock"];
  const TRANSPORT_LABEL: Record<DistillTransport, string> = {
    invoke: "IPC",
    http: "HTTP",
    mock: "MOCK",
  };
  let transport = $state<DistillTransport>(untrack(() => transportProp ?? distillTransport()));

  // ── Source resolution (follow the neighbouring markdown pane) ─────────────
  // The source is the active markdown tab in whichever pane isn't this Distill
  // tab. Distill's own pane has a `distill`-kind active tab, so it's skipped
  // naturally; we just take the first pane whose active tab is markdown.
  const sourceTab = $derived.by(() => {
    if (preview) return null;
    for (const pane of workspace.panes) {
      const active = pane.tabs.find((t) => t.id === pane.activeTabId);
      if (active?.kind === "markdown") return active;
    }
    return null;
  });
  const sourcePath = $derived(preview ? (previewSource ?? null) : (sourceTab?.path ?? null));
  // Bumps when the source switches, is saved (lastSavedTs) or is reloaded after
  // an external change (reloadToken) — drives a re-extract so the panel tracks
  // the note's saved content.
  const sourceVersion = $derived(
    sourceTab ? `${sourceTab.lastSavedTs ?? 0}:${sourceTab.reloadToken ?? 0}` : "",
  );

  // ── Extraction ────────────────────────────────────────────────────────────
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

  // Re-extract whenever the source path/version OR the transport changes.
  // Reading each here registers it as a dependency.
  $effect(() => {
    void transport;
    void sourceVersion;
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

<style>
  .distill {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    min-width: 0;
    min-height: 0;
    background: var(--color-base-100);
    color: var(--color-base-content);
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
