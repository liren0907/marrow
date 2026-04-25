import { commandsCtx } from "@milkdown/core";
import type { Ctx } from "@milkdown/ctx";
import { slashFactory, SlashProvider } from "@milkdown/plugin-slash";
import {
  wrapInBlockquoteCommand,
  wrapInBulletListCommand,
  wrapInHeadingCommand,
  wrapInOrderedListCommand,
} from "@milkdown/preset-commonmark";
import type { EditorView } from "@milkdown/prose/view";

export const slash = slashFactory("marrow-slash");

interface SlashItem {
  label: string;
  hint: string;
  keywords: string[];
  run: (ctx: Ctx) => void;
}

function buildItems(): SlashItem[] {
  return [
    {
      label: "Heading 1",
      hint: "#",
      keywords: ["heading", "h1", "heading1"],
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 1),
    },
    {
      label: "Heading 2",
      hint: "##",
      keywords: ["heading", "h2", "heading2"],
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 2),
    },
    {
      label: "Heading 3",
      hint: "###",
      keywords: ["heading", "h3", "heading3"],
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 3),
    },
    {
      label: "Bullet list",
      hint: "- item",
      keywords: ["bullet", "list", "ul", "unordered"],
      run: (c) => c.get(commandsCtx).call(wrapInBulletListCommand.key),
    },
    {
      label: "Numbered list",
      hint: "1. item",
      keywords: ["numbered", "ordered", "list", "ol"],
      run: (c) => c.get(commandsCtx).call(wrapInOrderedListCommand.key),
    },
    {
      label: "Quote",
      hint: "> text",
      keywords: ["quote", "blockquote"],
      run: (c) => c.get(commandsCtx).call(wrapInBlockquoteCommand.key),
    },
  ];
}

// Walk left from the cursor inside the current textblock to find the most
// recent `/` and return everything after it as the active slash query. Returns
// null if there's no `/` before the cursor in the same textblock or if any
// whitespace appears between `/` and cursor (queries don't span words).
function readSlashQuery(view: EditorView): string | null {
  const { $from } = view.state.selection;
  const textBefore = $from.parent.textBetween(
    0,
    $from.parentOffset,
    undefined,
    "￼",
  );
  const slashIdx = textBefore.lastIndexOf("/");
  if (slashIdx < 0) return null;
  const query = textBefore.slice(slashIdx + 1);
  if (/\s/.test(query)) return null;
  return query;
}

export function configSlash(ctx: Ctx): void {
  ctx.set(slash.key, {
    view: (view) => {
      const items = buildItems();
      let currentView = view;
      let filtered: SlashItem[] = items;
      let currentQuery = "";
      let selectedIdx = 0;
      let visible = false;

      const menuEl = document.createElement("div");
      menuEl.className = "marrow-slash-menu";

      let itemEls: HTMLButtonElement[] = [];

      function renderItems(): void {
        menuEl.innerHTML = "";
        itemEls = filtered.map((item, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.className = "marrow-slash-item";
          btn.innerHTML = `<span class="slash-label">${item.label}</span><span class="slash-hint">${item.hint}</span>`;
          btn.addEventListener("mousedown", (e) => {
            e.preventDefault();
            apply(i);
          });
          btn.addEventListener("mouseenter", () => select(i));
          menuEl.appendChild(btn);
          return btn;
        });
        applySelectedClass();
      }

      function applySelectedClass(): void {
        itemEls.forEach((el, idx) => {
          el.classList.toggle("selected", idx === selectedIdx);
        });
      }

      function select(i: number): void {
        if (filtered.length === 0) {
          selectedIdx = 0;
          return;
        }
        selectedIdx = ((i % filtered.length) + filtered.length) % filtered.length;
        applySelectedClass();
      }

      function refilter(query: string): void {
        currentQuery = query;
        const q = query.trim().toLowerCase();
        filtered = q.length === 0
          ? items
          : items.filter((item) =>
              item.keywords.some((k) => k.includes(q)),
            );
        selectedIdx = 0;
        renderItems();
        if (filtered.length === 0) {
          provider.hide();
        }
      }

      function apply(i: number): void {
        const item = filtered[i];
        if (!item) return;
        const { state, dispatch } = currentView;
        const { $from } = state.selection;
        // Delete the trigger "/" plus the typed query before the cursor.
        const start = Math.max(0, $from.pos - 1 - currentQuery.length);
        if (start < $from.pos) {
          dispatch(state.tr.delete(start, $from.pos));
        }
        item.run(ctx);
        provider.hide();
        currentView.focus();
      }

      const provider = new SlashProvider({
        content: menuEl,
        trigger: "/",
        debounce: 20,
        // Default shouldShow only fires when the last char before the cursor
        // is exactly `/`, which kills the menu the moment the user starts
        // typing a query. Override to keep it open while a `/...` token is
        // active in the current textblock.
        shouldShow: (view) => readSlashQuery(view) !== null,
      });

      const origShow = provider.onShow;
      const origHide = provider.onHide;
      provider.onShow = () => {
        visible = true;
        refilter(readSlashQuery(currentView) ?? "");
        origShow?.();
      };
      provider.onHide = () => {
        visible = false;
        origHide?.();
      };

      // Initial render so the menu has buttons before first show.
      renderItems();

      const onKey = (e: KeyboardEvent) => {
        if (!visible) return;
        if (e.key === "ArrowDown") {
          e.preventDefault();
          select(selectedIdx + 1);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          select(selectedIdx - 1);
        } else if (e.key === "Enter") {
          if (filtered.length === 0) return;
          e.preventDefault();
          apply(selectedIdx);
        } else if (e.key === "Escape") {
          e.preventDefault();
          provider.hide();
        }
      };
      window.addEventListener("keydown", onKey, true);

      return {
        update: (v, prev) => {
          currentView = v;
          provider.update(v, prev);
          if (visible) {
            const q = readSlashQuery(v);
            if (q === null) {
              provider.hide();
            } else if (q !== currentQuery) {
              refilter(q);
            }
          }
        },
        destroy: () => {
          window.removeEventListener("keydown", onKey, true);
          provider.destroy();
          menuEl.remove();
        },
      };
    },
  });
}
