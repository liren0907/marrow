import type { EditorView } from "@milkdown/prose/view";
import type { Schema } from "@milkdown/prose/model";
import { WIKI_LINK_RE, parseWikiInner } from "./regex";

interface Replacement {
  from: number;
  to: number;
  target: string;
  display: string;
}

export function convertTextToWikiLinks(view: EditorView): boolean {
  const schema: Schema = view.state.schema;
  const wikiLinkType = schema.nodes.wikiLink;
  if (!wikiLinkType) return false;

  const replacements: Replacement[] = [];
  view.state.doc.descendants((node, pos) => {
    if (!node.isText) return;
    const text = node.text ?? "";
    WIKI_LINK_RE.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = WIKI_LINK_RE.exec(text)) !== null) {
      // Skip transclusion syntax `![[target]]` — handled by transclusion module.
      if (m.index > 0 && text[m.index - 1] === "!") continue;
      const { target, display } = parseWikiInner(m[1]);
      if (!target) continue;
      const from = pos + m.index;
      const to = from + m[0].length;
      replacements.push({ from, to, target, display });
    }
  });

  if (replacements.length === 0) return false;

  // Apply in reverse order so earlier positions don't shift.
  let tr = view.state.tr;
  for (let i = replacements.length - 1; i >= 0; i--) {
    const r = replacements[i];
    tr = tr.replaceWith(
      r.from,
      r.to,
      wikiLinkType.create({ target: r.target, display: r.display }),
    );
  }
  tr.setMeta("addToHistory", false);
  tr.setMeta("marrow-wikilink-loadpass", true);
  view.dispatch(tr);
  return true;
}
