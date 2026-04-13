<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { workspace } from "$lib/workspace/workspace.svelte";
  import { backlinks } from "$lib/workspace/backlinkIndex.svelte";
  import type { Tab } from "$lib/workspace/types";

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let { tab }: { tab: Tab } = $props();

  let host: HTMLDivElement;
  // IMPORTANT: cytoscape Core handle MUST live in a plain `let` (not $state).
  // Svelte 5's proxy will deep-observe internal handles and corrupt them.
  // Same rule as Milkdown Editor / ProseMirror EditorView / CodeMirror EditorView.
  let cy: cytoscape.Core | null = null;
  let themeObserver: MutationObserver | null = null;
  let lastFileIndexLength = 0;
  let lastBacklinksBuilt = 0;

  function buildGraphData(): {
    nodes: cytoscape.ElementDefinition[];
    edges: cytoscape.ElementDefinition[];
  } {
    const mdFiles = workspace.fileIndex.filter((f) => f.kind === "markdown");
    const nodes: cytoscape.ElementDefinition[] = mdFiles.map((f) => ({
      data: {
        id: f.path,
        label: f.name.replace(/\.md$/i, ""),
      },
    }));
    const edges: cytoscape.ElementDefinition[] = [];
    let edgeId = 0;
    for (const [, entries] of backlinks.byTarget) {
      for (const entry of entries) {
        const targetPath = workspace.resolveBasename(entry.target);
        if (!targetPath || targetPath === entry.sourcePath) continue;
        edges.push({
          data: {
            id: `e${edgeId++}`,
            source: entry.sourcePath,
            target: targetPath,
          },
        });
      }
    }
    return { nodes, edges };
  }

  function styleFromTheme(): cytoscape.StylesheetStyle[] {
    const root = getComputedStyle(document.documentElement);
    const bc = root.getPropertyValue("--bc").trim() || "0 0 0";
    const p = root.getPropertyValue("--p").trim() || "0.6 0.2 250";
    const b3 = root.getPropertyValue("--b3").trim() || "0.85 0 0";
    return [
      {
        selector: "node",
        style: {
          "background-color": `oklch(${p})`,
          label: "data(label)",
          color: `oklch(${bc})`,
          "font-size": "10px",
          "text-valign": "bottom",
          "text-halign": "center",
          "text-margin-y": 4,
          width: 14,
          height: 14,
          "border-width": 0,
        },
      },
      {
        selector: "node.highlighted",
        style: {
          "border-width": 2,
          "border-color": `oklch(${bc})`,
        },
      },
      {
        selector: "edge",
        style: {
          width: 1,
          "line-color": `oklch(${b3})`,
          "target-arrow-color": `oklch(${b3})`,
          "target-arrow-shape": "triangle",
          "curve-style": "bezier",
          "arrow-scale": 0.8,
        },
      },
    ];
  }

  function rebuild(): void {
    if (!cy) return;
    const { nodes, edges } = buildGraphData();
    cy.batch(() => {
      cy!.elements().remove();
      cy!.add([...nodes, ...edges]);
    });
    cy.layout({ name: "cose", animate: false, idealEdgeLength: () => 80 } as cytoscape.LayoutOptions).run();
  }

  onMount(() => {
    const { nodes, edges } = buildGraphData();
    cy = cytoscape({
      container: host,
      elements: [...nodes, ...edges],
      style: styleFromTheme(),
      layout: { name: "cose", animate: false, idealEdgeLength: () => 80 } as cytoscape.LayoutOptions,
      wheelSensitivity: 0.2,
      minZoom: 0.2,
      maxZoom: 3,
    });

    cy.on("tap", "node", (e) => {
      const path = e.target.id() as string;
      workspace.replaceCurrentTab(path);
    });
    cy.on("mouseover", "node", (e) => {
      const node = e.target;
      node.addClass("highlighted");
      node.connectedEdges().connectedNodes().addClass("highlighted");
    });
    cy.on("mouseout", "node", () => {
      cy?.nodes().removeClass("highlighted");
    });

    lastFileIndexLength = workspace.fileIndex.length;
    lastBacklinksBuilt = backlinks.lastBuilt;

    themeObserver = new MutationObserver(() => {
      if (!cy) return;
      cy.style(styleFromTheme());
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
  });

  onDestroy(() => {
    themeObserver?.disconnect();
    themeObserver = null;
    cy?.destroy();
    cy = null;
  });

  // Rebuild graph when fileIndex changes or backlinks index gets rebuilt.
  // Touch the reactive deps explicitly so Svelte tracks them.
  $effect(() => {
    const len = workspace.fileIndex.length;
    const built = backlinks.lastBuilt;
    if (!cy) return;
    if (len !== lastFileIndexLength || built !== lastBacklinksBuilt) {
      lastFileIndexLength = len;
      lastBacklinksBuilt = built;
      rebuild();
    }
  });
</script>

<div bind:this={host} class="w-full h-full bg-base-100"></div>
