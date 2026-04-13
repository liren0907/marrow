import { $inputRule } from "@milkdown/utils";
import { InputRule } from "@milkdown/prose/inputrules";
import { wikiLinkSchema } from "./node";

export const wikiLinkInputRule = $inputRule((ctx) => {
  return new InputRule(/\[\[([^\[\]\n]+)\]\]$/, (state, match, start, end) => {
    const target = match[1]?.trim();
    if (!target) return null;
    // Don't fire if cursor is already inside a wikiLink (paranoia: atom nodes
    // shouldn't accept text input, but better safe than corrupt).
    const $start = state.doc.resolve(start);
    if ($start.parent.type.name === "wikiLink") return null;
    return state.tr.replaceWith(
      start,
      end,
      wikiLinkSchema.type(ctx).create({ target }),
    );
  });
});
