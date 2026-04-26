import { $ctx, $prose } from "@milkdown/utils";
import { Plugin, PluginKey } from "@milkdown/prose/state";
import { workspace } from "$lib/workspace/workspace.svelte";
import { readTextFile } from "$lib/workspace/tauri";
import { classifyFile } from "$lib/workspace/fileKind";
import { safeConvertFileSrc } from "$lib/utils/tauriUtils";
import { renderEmbedded, renderEmbeddedSection } from "./renderer";
import { advancedSettings } from "$lib/settings/advancedSettings.svelte";

export type TransclusionClickHandler = (target: string) => void;

export interface TransclusionConfig {
  onClick: TransclusionClickHandler | null;
}

export const transclusionConfigCtx = $ctx<TransclusionConfig, "transclusionConfig">(
  { onClick: null },
  "transclusionConfig",
);

const transclusionPluginKey = new PluginKey("marrow-transclusion-view");

// Module-level subscriber set so multiple NodeViews can react to the same
// fs-event without each one wiring its own global listener.
const subscribers = new Map<string, Set<() => void>>();

function subscribe(targetPath: string, fn: () => void): () => void {
  let set = subscribers.get(targetPath);
  if (!set) {
    set = new Set();
    subscribers.set(targetPath, set);
  }
  set.add(fn);
  return () => {
    set?.delete(fn);
    if (set?.size === 0) subscribers.delete(targetPath);
  };
}

// Called by fsEvents.ts when a markdown file's content changes.
export function notifyTransclusionTargets(targetPath: string): void {
  const set = subscribers.get(targetPath);
  if (!set) return;
  for (const fn of set) {
    try {
      fn();
    } catch {
      // ignore
    }
  }
}

function ancestorEmbedDepth(node: Node | null): number {
  let depth = 0;
  let cur: Node | null = node;
  while (cur && cur.nodeType === 1) {
    if ((cur as HTMLElement).classList?.contains("transclusion-embed")) {
      depth++;
    }
    cur = (cur as HTMLElement).parentNode;
  }
  return depth;
}

export const transclusionNodeView = $prose((ctx) => {
  return new Plugin({
    key: transclusionPluginKey,
    props: {
      nodeViews: {
        transclusion: (node, _view, _getPos) => {
          const target = node.attrs.target as string;
          const section = node.attrs.section as string;

          const dom = document.createElement("div");
          dom.dataset.transclusion = "";
          dom.dataset.target = target;
          dom.dataset.section = section;
          dom.className = "transclusion-embed";

          const header = document.createElement("div");
          header.className = "transclusion-header";
          const arrow = document.createElement("span");
          arrow.className = "material-symbols-rounded text-[14px]";
          arrow.textContent = "subdirectory_arrow_right";
          const label = document.createElement("span");
          label.className = "transclusion-target";
          label.textContent = section ? `${target} › ${section}` : target;
          header.append(arrow, label);
          header.addEventListener("mousedown", (e) => {
            e.preventDefault();
            e.stopPropagation();
            const cfg = ctx.get(transclusionConfigCtx.key);
            cfg.onClick?.(target);
          });

          const content = document.createElement("div");
          content.className = "transclusion-content";

          dom.append(header, content);

          let unsub: (() => void) | null = null;

          const render = async () => {
            // Depth check via DOM walk — must run AFTER mount, when the embed
            // is in the document and ancestors are visible.
            const depth = ancestorEmbedDepth(dom.parentNode);
            // Read live setting at render time so the user can tune the
            // limit from the Settings page; existing nodes will pick up
            // the new value on the next fs-event-driven re-render.
            if (depth >= advancedSettings.embedMaxDepth) {
              content.innerHTML =
                '<em class="transclusion-meta">[[recursive embed]]</em>';
              return;
            }

            const path = workspace.resolveBasename(target);
            if (!path) {
              dom.classList.add("unresolved");
              content.innerHTML =
                '<em class="transclusion-meta">unresolved</em>';
              return;
            }
            dom.classList.remove("unresolved");

            const kind = classifyFile(path);
            if (kind === "image") {
              const img = document.createElement("img");
              img.src = safeConvertFileSrc(path);
              img.alt = target;
              img.className = "transclusion-image";
              content.replaceChildren(img);
              if (unsub) {
                unsub();
                unsub = null;
              }
              return;
            }
            if (kind === "video") {
              const video = document.createElement("video");
              video.controls = true;
              video.src = safeConvertFileSrc(path);
              video.className = "transclusion-video";
              content.replaceChildren(video);
              if (unsub) {
                unsub();
                unsub = null;
              }
              return;
            }
            if (kind === "audio") {
              const audio = document.createElement("audio");
              audio.controls = true;
              audio.src = safeConvertFileSrc(path);
              audio.className = "transclusion-audio";
              content.replaceChildren(audio);
              if (unsub) {
                unsub();
                unsub = null;
              }
              return;
            }
            if (kind !== "markdown") {
              content.innerHTML = `<em class="transclusion-meta">cannot embed ${kind} file — click header to open</em>`;
              if (unsub) {
                unsub();
                unsub = null;
              }
              return;
            }

            try {
              const result = await readTextFile(path);
              content.innerHTML = section
                ? renderEmbeddedSection(result.content, section)
                : renderEmbedded(result.content);
              // Subscribe (or re-subscribe) once we know the resolved path.
              if (unsub) unsub();
              unsub = subscribe(path, () => void render());
            } catch (e) {
              content.innerHTML = `<em class="transclusion-meta">failed to load: ${e instanceof Error ? e.message : String(e)}</em>`;
            }
          };

          // Render after mount so DOM ancestry is visible to the depth check.
          queueMicrotask(() => void render());

          return {
            dom,
            update: (newNode) => {
              if (newNode.type.name !== "transclusion") return false;
              if (newNode.attrs.target !== target) return false;
              if (newNode.attrs.section !== section) return false;
              return true;
            },
            destroy: () => {
              if (unsub) unsub();
              unsub = null;
            },
          };
        },
      },
    },
  });
});
