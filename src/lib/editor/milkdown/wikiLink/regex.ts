// Shared regex for matching `[[target]]` wiki-links.
//
// IMPORTANT: any consumer that walks raw markdown text MUST also skip matches
// preceded by `!`, since `![[target]]` is a transclusion and is parsed by a
// separate pipeline (`src/lib/editor/milkdown/transclusion/`).
//
// The wiki-link load-pass and the backlink indexer both apply this exclusion.
export const WIKI_LINK_RE = /\[\[([^\[\]\n]+)\]\]/g;

// Splits the captured inner string into target + optional display alias.
// `[[notes|看這裡]]` → { target: "notes", display: "看這裡" }
// `[[notes]]`        → { target: "notes", display: "" }
export function parseWikiInner(inner: string): {
  target: string;
  display: string;
} {
  const trimmed = inner.trim();
  const pipeIdx = trimmed.indexOf("|");
  if (pipeIdx < 0) return { target: trimmed, display: "" };
  return {
    target: trimmed.slice(0, pipeIdx).trim(),
    display: trimmed.slice(pipeIdx + 1).trim(),
  };
}
