import { $inputRule } from "@milkdown/utils";
import { InputRule } from "@milkdown/prose/inputrules";
import { transclusionSchema } from "./node";

export const transclusionInputRule = $inputRule((ctx) => {
  return new InputRule(/!\[\[([^\[\]\n]+)\]\]$/, (state, match, start, end) => {
    const target = match[1]?.trim();
    if (!target) return null;
    const $start = state.doc.resolve(start);
    if ($start.parent.type.name === "transclusion") return null;
    return state.tr.replaceWith(
      start,
      end,
      transclusionSchema.type(ctx).create({ target }),
    );
  });
});
