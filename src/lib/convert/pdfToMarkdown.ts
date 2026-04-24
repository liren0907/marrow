import type { TextItem } from "pdfjs-dist/types/src/display/api";
import { loadPdfDoc } from "$lib/pdf/loadPdfDoc";

type Line = {
  text: string;
  y: number;
  fontSize: number;
};

const HEADING_H1_RATIO = 1.8;
const HEADING_H2_RATIO = 1.3;
const HEADING_MAX_LEN = 80;
const PARAGRAPH_GAP_RATIO = 1.5;
const SAME_BASELINE_RATIO = 0.5;

const SCANNED_MESSAGE =
  "> This PDF appears to be scanned — text extraction yielded nothing.\n" +
  "> OCR is not yet supported.\n";

export async function pdfToMarkdown(path: string): Promise<string> {
  const doc = await loadPdfDoc(path);
  try {
    const allLines: Line[] = [];
    for (let p = 1; p <= doc.numPages; p++) {
      const page = await doc.getPage(p);
      try {
        const tc = await page.getTextContent();
        const items = (tc.items as TextItem[]).filter(
          (i) => typeof i?.str === "string",
        );
        items.sort((a, b) => {
          const ya = a.transform[5];
          const yb = b.transform[5];
          if (yb !== ya) return yb - ya;
          return a.transform[4] - b.transform[4];
        });
        const pageLines = groupIntoLines(items);
        allLines.push(...pageLines);
      } finally {
        page.cleanup();
      }
    }

    if (allLines.every((l) => !l.text.trim())) {
      return SCANNED_MESSAGE;
    }

    const bodyFont = medianFontSize(allLines);
    const md = emitMarkdown(allLines, bodyFont);
    return postProcess(md);
  } finally {
    try {
      await doc.destroy();
    } catch {
      // ignore destroy errors — we're done with the doc either way
    }
  }
}

function groupIntoLines(items: TextItem[]): Line[] {
  const lines: Line[] = [];
  let cur: Line | null = null;
  let curX: number | null = null;

  for (const item of items) {
    const y = item.transform[5];
    const x = item.transform[4];
    const fontSize = Math.abs(item.transform[0]) || Math.abs(item.height) || 10;
    const str = item.str;

    if (
      cur &&
      Math.abs(y - cur.y) < cur.fontSize * SAME_BASELINE_RATIO
    ) {
      // Same line — append, inserting a space if neither side has one
      // and the x gap suggests a word boundary.
      const needsSpace =
        str.length > 0 &&
        cur.text.length > 0 &&
        !cur.text.endsWith(" ") &&
        !str.startsWith(" ") &&
        curX !== null &&
        x - curX > cur.fontSize * 0.2;
      cur.text += (needsSpace ? " " : "") + str;
      // keep the largest font size seen on this line (headings can
      // have a trailing marker/numeral at body size)
      if (fontSize > cur.fontSize) cur.fontSize = fontSize;
      curX = x + (item.width ?? 0);
    } else {
      if (cur) lines.push(cur);
      cur = { text: str, y, fontSize };
      curX = x + (item.width ?? 0);
    }
  }
  if (cur) lines.push(cur);

  // Trim each line; drop fully empty ones. A PDF with vertical-only whitespace
  // items would otherwise produce noise.
  return lines
    .map((l) => ({ ...l, text: l.text.replace(/\s+/g, " ").trim() }))
    .filter((l) => l.text.length > 0);
}

function medianFontSize(lines: Line[]): number {
  const sizes = lines.map((l) => l.fontSize).sort((a, b) => a - b);
  if (sizes.length === 0) return 10;
  const mid = Math.floor(sizes.length / 2);
  return sizes.length % 2 === 0 ? (sizes[mid - 1] + sizes[mid]) / 2 : sizes[mid];
}

function emitMarkdown(lines: Line[], bodyFont: number): string {
  const parts: string[] = [];
  let prev: Line | null = null;

  for (const line of lines) {
    if (prev) {
      const gap = prev.y - line.y;
      if (gap > bodyFont * PARAGRAPH_GAP_RATIO) {
        parts.push("");
      }
    }
    parts.push(renderLine(line, bodyFont));
    prev = line;
  }
  return parts.join("\n");
}

function renderLine(line: Line, bodyFont: number): string {
  const isShort = line.text.length < HEADING_MAX_LEN;
  if (isShort && line.fontSize >= bodyFont * HEADING_H1_RATIO) {
    return `# ${line.text}`;
  }
  if (isShort && line.fontSize >= bodyFont * HEADING_H2_RATIO) {
    return `## ${line.text}`;
  }
  return line.text;
}

function postProcess(md: string): string {
  return md
    .replace(/ﬁ/g, "fi")
    .replace(/ﬂ/g, "fl")
    .replace(/ﬀ/g, "ff")
    .replace(/ﬃ/g, "ffi")
    .replace(/ﬄ/g, "ffl")
    .replace(/\n{3,}/g, "\n\n")
    .trim() + "\n";
}
