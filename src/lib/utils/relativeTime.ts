// Relative time formatting for chrome / meta UI ("edited 2 days ago" etc).
//
// Hand-rolled rather than Intl.RelativeTimeFormat — we want full control over
// thresholds and verbiage (e.g. "just now" instead of "in 0 seconds"), and
// the pluralization rules are trivial in English. Add an i18n layer here if
// we ever need other languages.
//
// Both functions accept either second-level (Rust fs::metadata mtime) or
// millisecond-level (JS Date.now) timestamps and auto-detect — values below
// 1e12 are treated as seconds. Same heuristic that EditorMetaHeader was
// already using inline.

function toMs(ts: number): number {
  return ts < 1e12 ? ts * 1000 : ts;
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}

export function relativeTime(ts: number, now: number = Date.now()): string {
  const ms = toMs(ts);
  const diffSec = Math.max(0, Math.round((now - ms) / 1000));

  if (diffSec < 45) return "just now";
  if (diffSec < 90) return "1 min ago";

  const diffMin = Math.round(diffSec / 60);
  if (diffMin < 60) return `${plural(diffMin, "min")} ago`;

  const diffHr = Math.round(diffMin / 60);
  if (diffHr < 24) return `${plural(diffHr, "hr")} ago`;

  const diffDay = Math.round(diffHr / 24);
  if (diffDay < 30) return `${plural(diffDay, "day")} ago`;

  const diffMonth = Math.round(diffDay / 30);
  if (diffMonth < 12) return `${plural(diffMonth, "month")} ago`;

  const diffYear = Math.round(diffDay / 365);
  return `${plural(diffYear, "year")} ago`;
}

// Full-precision timestamp for tooltips: "2026-04-25 14:32:18".
export function formatFullDateTime(ts: number): string {
  const d = new Date(toMs(ts));
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
    `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
  );
}
