import type { FileKind } from "./types";

const MARKDOWN = new Set(["md", "markdown", "mdx"]);
const IMAGE = new Set([
  "png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "tiff", "avif",
]);
const VIDEO = new Set(["mp4", "webm", "mov", "avi", "mkv", "m4v"]);
const AUDIO = new Set(["mp3", "wav", "ogg", "flac", "m4a", "aac"]);
const PDF = new Set(["pdf"]);
const TEXT = new Set([
  "txt", "json", "yaml", "yml", "toml", "xml", "csv", "log", "ini", "conf",
  "sh", "bash", "zsh", "js", "ts", "jsx", "tsx", "mjs", "cjs",
  "py", "rb", "go", "rs", "c", "cpp", "h", "hpp", "java", "kt", "swift",
  "php", "html", "htm", "css", "scss", "less", "vue", "svelte",
]);

export function classifyFile(path: string): FileKind {
  // Guard: synthetic marrow:// paths (e.g. graph tab) should never be
  // misclassified by extension matching.
  if (path.startsWith("marrow://")) return "unsupported";
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  if (MARKDOWN.has(ext)) return "markdown";
  if (IMAGE.has(ext)) return "image";
  if (VIDEO.has(ext)) return "video";
  if (AUDIO.has(ext)) return "audio";
  if (PDF.has(ext)) return "pdf";
  if (TEXT.has(ext)) return "text";
  return "unsupported";
}

// Extensions for which `uvx markitdown` can produce useful Markdown output.
// `.md` is intentionally excluded (conversion is a no-op).
const CONVERTIBLE = new Set([
  "pdf",
  "docx",
  "pptx",
  "xlsx",
  "xls",
  "html",
  "htm",
  "epub",
  "ipynb",
  "csv",
  "json",
  "xml",
  "msg",
  "eml",
  "zip",
]);

export function isConvertible(path: string): boolean {
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  return CONVERTIBLE.has(ext);
}

export function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

export function dirname(path: string): string {
  const idx = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return idx > 0 ? path.slice(0, idx) : "";
}

export function joinPath(dir: string, name: string): string {
  if (!dir) return name;
  const sep = dir.includes("\\") && !dir.includes("/") ? "\\" : "/";
  return dir.endsWith("/") || dir.endsWith("\\") ? `${dir}${name}` : `${dir}${sep}${name}`;
}
