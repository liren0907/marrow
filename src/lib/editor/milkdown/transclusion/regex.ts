// Matches `![[target]]` transclusion syntax.
// The leading `!` is what distinguishes a transclusion from a wiki-link;
// the wiki-link load-pass and indexer must skip matches preceded by `!`.
export const TRANSCLUSION_RE = /!\[\[([^\[\]\n]+)\]\]/g;

// Splits the captured inner string into target + optional section.
// `![[notes#Setup]]` → { target: "notes", section: "Setup" }
// `![[notes]]`      → { target: "notes", section: "" }
export function parseTransclusionInner(inner: string): {
  target: string;
  section: string;
} {
  const trimmed = inner.trim();
  const hashIdx = trimmed.indexOf("#");
  if (hashIdx < 0) return { target: trimmed, section: "" };
  return {
    target: trimmed.slice(0, hashIdx).trim(),
    section: trimmed.slice(hashIdx + 1).trim(),
  };
}
