// Matches `![[target]]` transclusion syntax.
// The leading `!` is what distinguishes a transclusion from a wiki-link;
// the wiki-link load-pass and indexer must skip matches preceded by `!`.
export const TRANSCLUSION_RE = /!\[\[([^\[\]\n]+)\]\]/g;
