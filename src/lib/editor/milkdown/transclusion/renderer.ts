import MarkdownIt from "markdown-it";
import { advancedSettings } from "$lib/settings/advancedSettings.svelte";

// Singleton renderer for transclusion embeds.
// `html: false` is critical — innerHTML injection means we cannot allow
// arbitrary HTML from source markdown (XSS).
const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
});

export function renderEmbedded(text: string): string {
  const cap = advancedSettings.embedRenderBytes;
  if (text.length > cap) {
    return `<em>Embed too large to render (>${Math.round(cap / 1024)}KB)</em>`;
  }
  return md.render(text);
}

function normalizeHeading(text: string): string {
  return text.replace(/[*_`]/g, "").trim().toLowerCase();
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export function renderEmbeddedSection(
  text: string,
  headingText: string,
): string {
  const cap = advancedSettings.embedRenderBytes;
  if (text.length > cap) {
    return `<em>Embed too large to render (>${Math.round(cap / 1024)}KB)</em>`;
  }
  const target = normalizeHeading(headingText);
  const tokens = md.parse(text, {});
  const lines = text.split("\n");

  let startLine = -1;
  let endLine = lines.length;
  let level = 0;

  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i];
    if (t.type !== "heading_open" || !t.map) continue;
    if (startLine < 0) {
      const inline = tokens[i + 1];
      const headingContent = inline?.content ?? "";
      if (normalizeHeading(headingContent) === target) {
        startLine = t.map[0];
        level = parseInt(t.tag.slice(1), 10);
      }
    } else {
      const nextLevel = parseInt(t.tag.slice(1), 10);
      if (nextLevel <= level) {
        endLine = t.map[0];
        break;
      }
    }
  }

  if (startLine < 0) {
    return `<em class="transclusion-meta">section "${escapeHtml(headingText)}" not found</em>`;
  }
  const slice = lines.slice(startLine, endLine).join("\n");
  return md.render(slice);
}
