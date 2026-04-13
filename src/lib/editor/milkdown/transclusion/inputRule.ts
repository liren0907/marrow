import { $inputRule } from "@milkdown/utils";
import { InputRule } from "@milkdown/prose/inputrules";
import { transclusionSchema } from "./node";
import { parseTransclusionInner } from "./regex";

export const transclusionInputRule = $inputRule((ctx) => {
  return new InputRule(/!\[\[([^\[\]\n]+)\]\]$/, (state, match, start, end) => {
    const inner = match[1];
    if (!inner) return null;
    const { target, section } = parseTransclusionInner(inner);
    if (!target) return null;
    const $start = state.doc.resolve(start);
    if ($start.parent.type.name === "transclusion") return null;
    return state.tr.replaceWith(
      start,
      end,
      transclusionSchema.type(ctx).create({ target, section }),
    );
  });
});
