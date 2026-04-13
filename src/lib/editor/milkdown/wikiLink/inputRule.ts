import { $inputRule } from "@milkdown/utils";
import { InputRule } from "@milkdown/prose/inputrules";
import { wikiLinkSchema } from "./node";
import { parseWikiInner } from "./regex";

export const wikiLinkInputRule = $inputRule((ctx) => {
  return new InputRule(/\[\[([^\[\]\n]+)\]\]$/, (state, match, start, end) => {
    const inner = match[1];
    if (!inner) return null;
    const { target, display } = parseWikiInner(inner);
    if (!target) return null;
    // Don't fire if cursor is already inside a wikiLink (paranoia: atom nodes
    // shouldn't accept text input, but better safe than corrupt).
    const $start = state.doc.resolve(start);
    if ($start.parent.type.name === "wikiLink") return null;
    return state.tr.replaceWith(
      start,
      end,
      wikiLinkSchema.type(ctx).create({ target, display }),
    );
  });
});
