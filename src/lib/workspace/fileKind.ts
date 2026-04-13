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
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  if (MARKDOWN.has(ext)) return "markdown";
  if (IMAGE.has(ext)) return "image";
  if (VIDEO.has(ext)) return "video";
  if (AUDIO.has(ext)) return "audio";
  if (PDF.has(ext)) return "pdf";
  if (TEXT.has(ext)) return "text";
  return "unsupported";
}

export function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}
