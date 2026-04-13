import type { Extension } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { markdown } from "@codemirror/lang-markdown";

const map: Record<string, () => Extension> = {
  ts: () => javascript({ typescript: true }),
  tsx: () => javascript({ typescript: true, jsx: true }),
  js: () => javascript(),
  jsx: () => javascript({ jsx: true }),
  mjs: () => javascript(),
  cjs: () => javascript(),
  json: () => json(),
  py: () => python(),
  rs: () => rust(),
  md: () => markdown(),
  markdown: () => markdown(),
  mdx: () => markdown(),
};

export function languageFor(path: string): Extension | [] {
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  return map[ext]?.() ?? [];
}
