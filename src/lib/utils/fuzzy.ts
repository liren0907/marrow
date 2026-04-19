// Subsequence fuzzy matcher with positional + word-boundary scoring.

const SEPARATOR_RE = /[\s\-_./\\]/;

export interface MatchResult {
  score: number;
  indices: number[];
}

function isWordBoundary(text: string, i: number): boolean {
  if (i === 0) return true;
  return SEPARATOR_RE.test(text[i - 1]);
}

export function fuzzyMatch(query: string, text: string): MatchResult | null {
  if (!query) return { score: 1, indices: [] };
  const q = query.toLowerCase();
  const t = text.toLowerCase();

  const indices: number[] = [];
  let qi = 0;
  let score = 0;
  let prevMatch = -2;
  for (let i = 0; i < t.length && qi < q.length; i++) {
    if (t[i] === q[qi]) {
      indices.push(i);
      let s = 1;
      if (isWordBoundary(t, i)) s += 3;
      if (i === prevMatch + 1) s += 2;
      score += s;
      prevMatch = i;
      qi++;
    }
  }
  if (qi < q.length) return null;
  // Reward shorter matches (closer to start of string).
  score += Math.max(0, 20 - indices[0]);
  return { score, indices };
}
