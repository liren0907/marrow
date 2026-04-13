import { slashFactory, SlashProvider } from "@milkdown/plugin-slash";
import type { Ctx } from "@milkdown/ctx";
import { workspace } from "$lib/workspace/workspace.svelte";
import { readTextFile } from "$lib/workspace/tauri";

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
const HEADING_LINE_RE = /^(#{1,6})\s+(.+?)\s*$/gm;

interface HeadingItem {
  text: string;
  level: number;
}

export function configTransclusionSuggest(
  ctx: Ctx,
  getSuggestions: TransclusionSuggestionSource,
): void {
  ctx.set(transclusionSuggest.key, {
    view: (view) => {
      let currentView = view;
      let currentItems: TransclusionSuggestion[] = [];
      let headingItems: HeadingItem[] = [];
      let mode: "file" | "heading" = "file";
      let headingFilename = "";
      let selectedIdx = 0;
      let visible = false;
      let queryStart = 0;
      let headingFetchToken = 0;

      const menuEl = document.createElement("div");
      menuEl.className = "marrow-wikilink-menu";

      function render(): void {
        menuEl.innerHTML = "";
        if (mode === "heading") {
          if (headingItems.length === 0) {
            const empty = document.createElement("div");
            empty.className = "marrow-wikilink-empty";
            empty.textContent = "No headings";
            menuEl.appendChild(empty);
            return;
          }
          headingItems.forEach((item, i) => {
            const btn = document.createElement("button");
            btn.type = "button";
            btn.className = "marrow-wikilink-item";
            if (i === selectedIdx) btn.classList.add("selected");
            const label = document.createElement("span");
            label.className = "wl-name";
            label.textContent = "# ".repeat(item.level) + item.text;
            btn.append(label);
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
          return;
        }
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

      async function loadHeadings(filename: string): Promise<HeadingItem[]> {
        const path = workspace.resolveBasename(filename);
        if (!path) return [];
        try {
          const result = await readTextFile(path);
          const out: HeadingItem[] = [];
          HEADING_LINE_RE.lastIndex = 0;
          let m: RegExpExecArray | null;
          while ((m = HEADING_LINE_RE.exec(result.content)) !== null) {
            out.push({ level: m[1].length, text: m[2].trim() });
          }
          return out;
        } catch {
          return [];
        }
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
          headingItems = [];
          return;
        }
        const query = m[1];
        queryStart = $from.pos - m[0].length; // position of the `!`

        const hashIdx = query.indexOf("#");
        if (hashIdx >= 0) {
          // Heading mode: query = "filename#headingPrefix"
          mode = "heading";
          const filename = query.slice(0, hashIdx).trim().replace(/\.md$/i, "");
          const headingPrefix = query.slice(hashIdx + 1).trim().toLowerCase();
          headingFilename = filename;

          const myToken = ++headingFetchToken;
          void loadHeadings(filename).then((items) => {
            if (myToken !== headingFetchToken) return;
            headingItems = headingPrefix
              ? items.filter((h) => h.text.toLowerCase().includes(headingPrefix))
              : items;
            headingItems = headingItems.slice(0, 8);
            if (selectedIdx >= headingItems.length) selectedIdx = 0;
            if (visible) render();
          });
          return;
        }

        mode = "file";
        const stem = query.replace(/\.md$/i, "");
        currentItems = getSuggestions(stem).slice(0, 8);
        if (selectedIdx >= currentItems.length) selectedIdx = 0;
      }

      function apply(i: number): void {
        const transclusionType =
          currentView.state.schema.nodes.transclusion;
        if (!transclusionType) return;

        if (mode === "heading") {
          const item = headingItems[i];
          if (!item) return;
          let tr = currentView.state.tr.delete(
            queryStart,
            currentView.state.selection.from,
          );
          tr = tr.insert(
            queryStart,
            transclusionType.create({
              target: headingFilename,
              section: item.text,
            }),
          );
          currentView.dispatch(tr);
          provider.hide();
          currentView.focus();
          return;
        }

        const item = currentItems[i];
        if (!item) return;
        const target = item.name.replace(/\.md$/i, "");
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
        const len =
          mode === "heading" ? headingItems.length : currentItems.length;
        if (e.key === "ArrowDown") {
          e.preventDefault();
          selectedIdx = (selectedIdx + 1) % Math.max(1, len);
          render();
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          selectedIdx =
            (selectedIdx - 1 + Math.max(1, len)) % Math.max(1, len);
          render();
        } else if (e.key === "Enter") {
          if (len === 0) return;
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
