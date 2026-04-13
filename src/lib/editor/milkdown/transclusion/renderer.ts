import MarkdownIt from "markdown-it";

// Singleton renderer for transclusion embeds.
// `html: false` is critical — innerHTML injection means we cannot allow
// arbitrary HTML from source markdown (XSS).
const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
});

const MAX_RENDER_BYTES = 100 * 1024;

export function renderEmbedded(text: string): string {
  if (text.length > MAX_RENDER_BYTES) {
    return `<em>Embed too large to render (>${MAX_RENDER_BYTES / 1024}KB)</em>`;
  }
  return md.render(text);
}
