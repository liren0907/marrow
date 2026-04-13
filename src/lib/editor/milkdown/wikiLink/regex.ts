// Shared regex for matching `[[target]]` wiki-links.
//
// IMPORTANT: any consumer that walks raw markdown text MUST also skip matches
// preceded by `!`, since `![[target]]` is a transclusion and is parsed by a
// separate pipeline (`src/lib/editor/milkdown/transclusion/`).
//
// The wiki-link load-pass and the backlink indexer both apply this exclusion.
export const WIKI_LINK_RE = /\[\[([^\[\]\n]+)\]\]/g;
