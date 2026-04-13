import { commandsCtx } from "@milkdown/core";
import type { Ctx } from "@milkdown/ctx";
import { slashFactory, SlashProvider } from "@milkdown/plugin-slash";
import {
  wrapInBlockquoteCommand,
  wrapInBulletListCommand,
  wrapInHeadingCommand,
  wrapInOrderedListCommand,
} from "@milkdown/preset-commonmark";

export const slash = slashFactory("marrow-slash");

interface SlashItem {
  label: string;
  hint: string;
  run: (ctx: Ctx) => void;
}

function buildItems(): SlashItem[] {
  return [
    {
      label: "Heading 1",
      hint: "#",
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 1),
    },
    {
      label: "Heading 2",
      hint: "##",
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 2),
    },
    {
      label: "Heading 3",
      hint: "###",
      run: (c) => c.get(commandsCtx).call(wrapInHeadingCommand.key, 3),
    },
    {
      label: "Bullet list",
      hint: "- item",
      run: (c) => c.get(commandsCtx).call(wrapInBulletListCommand.key),
    },
    {
      label: "Numbered list",
      hint: "1. item",
      run: (c) => c.get(commandsCtx).call(wrapInOrderedListCommand.key),
    },
    {
      label: "Quote",
      hint: "> text",
      run: (c) => c.get(commandsCtx).call(wrapInBlockquoteCommand.key),
    },
  ];
}

export function configSlash(ctx: Ctx): void {
  ctx.set(slash.key, {
    view: (view) => {
      const items = buildItems();
      let currentView = view;
      let selectedIdx = 0;
      let visible = false;

      const menuEl = document.createElement("div");
      menuEl.className = "marrow-slash-menu";

      const itemEls: HTMLButtonElement[] = items.map((item, i) => {
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

      function select(i: number): void {
        selectedIdx = ((i % items.length) + items.length) % items.length;
        itemEls.forEach((el, idx) => {
          el.classList.toggle("selected", idx === selectedIdx);
        });
      }
      select(0);

      function apply(i: number): void {
        const item = items[i];
        const { state, dispatch } = currentView;
        const { $from } = state.selection;
        // Delete the trigger "/" char before the cursor.
        const start = Math.max(0, $from.pos - 1);
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
      });

      const origShow = provider.onShow;
      const origHide = provider.onHide;
      provider.onShow = () => {
        visible = true;
        select(0);
        origShow?.();
      };
      provider.onHide = () => {
        visible = false;
        origHide?.();
      };

      const onKey = (e: KeyboardEvent) => {
        if (!visible) return;
        if (e.key === "ArrowDown") {
          e.preventDefault();
          select(selectedIdx + 1);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          select(selectedIdx - 1);
        } else if (e.key === "Enter") {
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
