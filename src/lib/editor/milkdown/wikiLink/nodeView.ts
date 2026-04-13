import { $ctx, $prose } from "@milkdown/utils";
import { Plugin, PluginKey } from "@milkdown/prose/state";

export type WikiLinkClickHandler = (
  target: string,
  mods: { meta: boolean; shift: boolean; middle: boolean },
) => void;

export interface WikiLinkConfig {
  onClick: WikiLinkClickHandler | null;
  isResolved: (target: string) => boolean;
}

export const wikiLinkConfigCtx = $ctx<WikiLinkConfig, "wikiLinkConfig">(
  { onClick: null, isResolved: () => true },
  "wikiLinkConfig",
);

const wikiLinkPluginKey = new PluginKey("marrow-wikilink-view");

export const wikiLinkNodeView = $prose((ctx) => {
  return new Plugin({
    key: wikiLinkPluginKey,
    props: {
      nodeViews: {
        wikiLink: (node) => {
          const dom = document.createElement("span");
          dom.dataset.wikiLink = "";
          dom.dataset.target = node.attrs.target as string;
          dom.className = "wiki-link";
          dom.textContent = node.attrs.target as string;

          const refresh = () => {
            const cfg = ctx.get(wikiLinkConfigCtx.key);
            const resolved = cfg.isResolved(node.attrs.target as string);
            dom.classList.toggle("unresolved", !resolved);
          };
          refresh();

          const handleMouseDown = (e: MouseEvent) => {
            // Only intercept left + middle clicks.
            if (e.button !== 0 && e.button !== 1) return;
            e.preventDefault();
            e.stopPropagation();
            const cfg = ctx.get(wikiLinkConfigCtx.key);
            cfg.onClick?.(node.attrs.target as string, {
              meta: e.metaKey || e.ctrlKey,
              shift: e.shiftKey,
              middle: e.button === 1,
            });
          };
          dom.addEventListener("mousedown", handleMouseDown);

          return {
            dom,
            update: (newNode) => {
              if (newNode.type.name !== "wikiLink") return false;
              if (newNode.attrs.target !== node.attrs.target) {
                dom.textContent = newNode.attrs.target as string;
                dom.dataset.target = newNode.attrs.target as string;
              }
              refresh();
              return true;
            },
            destroy: () => {
              dom.removeEventListener("mousedown", handleMouseDown);
            },
          };
        },
      },
    },
  });
});
