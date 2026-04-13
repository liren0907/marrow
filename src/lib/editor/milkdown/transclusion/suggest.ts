import { slashFactory, SlashProvider } from "@milkdown/plugin-slash";
import type { Ctx } from "@milkdown/ctx";

export const transclusionSuggest = slashFactory("marrow-transclusion-suggest");

export interface TransclusionSuggestion {
  name: string;
  path: string;
  folder: string;
}

export type TransclusionSuggestionSource = (
  query: string,
) => TransclusionSuggestion[];

const SUGGEST_TRIGGER = /!\[\[([^\]\n]*)$/;

export function configTransclusionSuggest(
  ctx: Ctx,
  getSuggestions: TransclusionSuggestionSource,
): void {
  ctx.set(transclusionSuggest.key, {
    view: (view) => {
      let currentView = view;
      let currentItems: TransclusionSuggestion[] = [];
      let selectedIdx = 0;
      let visible = false;
      let queryStart = 0;

      const menuEl = document.createElement("div");
      menuEl.className = "marrow-wikilink-menu";

      function render(): void {
        menuEl.innerHTML = "";
        if (currentItems.length === 0) {
          const empty = document.createElement("div");
          empty.className = "marrow-wikilink-empty";
          empty.textContent = "No matches";
          menuEl.appendChild(empty);
          return;
        }
        currentItems.forEach((item, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.className = "marrow-wikilink-item";
          if (i === selectedIdx) btn.classList.add("selected");
          const label = document.createElement("span");
          label.className = "wl-name";
          label.textContent = "↪ " + item.name.replace(/\.md$/i, "");
          const folder = document.createElement("span");
          folder.className = "wl-folder";
          folder.textContent = item.folder;
          btn.append(label, folder);
          btn.addEventListener("mousedown", (e) => {
            e.preventDefault();
            apply(i);
          });
          btn.addEventListener("mouseenter", () => {
            selectedIdx = i;
            render();
          });
          menuEl.appendChild(btn);
        });
      }

      function refreshFromState(): void {
        const { state } = currentView;
        const { $from } = state.selection;
        const before = $from.parent.textBetween(
          Math.max(0, $from.parentOffset - 200),
          $from.parentOffset,
          undefined,
          "\ufffc",
        );
        const m = SUGGEST_TRIGGER.exec(before);
        if (!m) {
          currentItems = [];
          return;
        }
        const query = m[1];
        queryStart = $from.pos - m[0].length; // position of the `!`
        const stem = query.replace(/\.md$/i, "");
        currentItems = getSuggestions(stem).slice(0, 8);
        if (selectedIdx >= currentItems.length) selectedIdx = 0;
      }

      function apply(i: number): void {
        const item = currentItems[i];
        if (!item) return;
        const target = item.name.replace(/\.md$/i, "");
        const transclusionType =
          currentView.state.schema.nodes.transclusion;
        if (!transclusionType) return;
        // Delete the `![[query` text first, then insert the transclusion node.
        let tr = currentView.state.tr.delete(
          queryStart,
          currentView.state.selection.from,
        );
        tr = tr.insert(queryStart, transclusionType.create({ target }));
        currentView.dispatch(tr);
        provider.hide();
        currentView.focus();
      }

      const provider = new SlashProvider({
        content: menuEl,
        trigger: ["[", "]", "!"],
        debounce: 20,
        shouldShow: (v) => {
          const { state } = v;
          const { $from } = state.selection;
          if (!state.selection.empty) return false;
          const before = $from.parent.textBetween(
            Math.max(0, $from.parentOffset - 200),
            $from.parentOffset,
            undefined,
            "\ufffc",
          );
          return SUGGEST_TRIGGER.test(before);
        },
      });

      const origShow = provider.onShow;
      const origHide = provider.onHide;
      provider.onShow = () => {
        visible = true;
        selectedIdx = 0;
        refreshFromState();
        render();
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
          selectedIdx = (selectedIdx + 1) % Math.max(1, currentItems.length);
          render();
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          selectedIdx =
            (selectedIdx - 1 + Math.max(1, currentItems.length)) %
            Math.max(1, currentItems.length);
          render();
        } else if (e.key === "Enter") {
          if (currentItems.length === 0) return;
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
            refreshFromState();
            render();
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
