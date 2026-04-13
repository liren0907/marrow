import type { EditorView } from "@milkdown/prose/view";
import type { Schema } from "@milkdown/prose/model";
import { TRANSCLUSION_RE, parseTransclusionInner } from "./regex";

interface Replacement {
  from: number;
  to: number;
  target: string;
  section: string;
}

export function convertTextToTransclusions(view: EditorView): boolean {
  const schema: Schema = view.state.schema;
  const transclusionType = schema.nodes.transclusion;
  if (!transclusionType) return false;

  const replacements: Replacement[] = [];
  view.state.doc.descendants((node, pos) => {
    if (!node.isText) return;
    const text = node.text ?? "";
    TRANSCLUSION_RE.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = TRANSCLUSION_RE.exec(text)) !== null) {
      const { target, section } = parseTransclusionInner(m[1]);
      if (!target) continue;
      const from = pos + m.index;
      const to = from + m[0].length;
      replacements.push({ from, to, target, section });
    }
  });

  if (replacements.length === 0) return false;

  let tr = view.state.tr;
  for (let i = replacements.length - 1; i >= 0; i--) {
    const r = replacements[i];
    // transclusion is a block atom — we must replace with a block-level node,
    // which means splitting the parent paragraph if needed. ProseMirror's
    // replaceWith handles that as long as the replacement is inserted at a
    // valid block position. For inline text positions, we delete and insert
    // separately so the doc structure stays valid.
    tr = tr.delete(r.from, r.to);
    tr = tr.insert(
      r.from,
      transclusionType.create({ target: r.target, section: r.section }),
    );
  }
  tr.setMeta("addToHistory", false);
  tr.setMeta("marrow-transclusion-loadpass", true);
  view.dispatch(tr);
  return true;
}
