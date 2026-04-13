import type { Ctx } from "@milkdown/ctx";
import { prismConfig } from "@milkdown/plugin-prism";

import bash from "refractor/bash";
import c from "refractor/c";
import cpp from "refractor/cpp";
import css from "refractor/css";
import go from "refractor/go";
import java from "refractor/java";
import javascript from "refractor/javascript";
import json from "refractor/json";
import jsx from "refractor/jsx";
import markdown from "refractor/markdown";
import markup from "refractor/markup";
import python from "refractor/python";
import rust from "refractor/rust";
import toml from "refractor/toml";
import tsx from "refractor/tsx";
import typescript from "refractor/typescript";
import yaml from "refractor/yaml";

const LANGUAGES = [
  bash,
  c,
  cpp,
  css,
  go,
  java,
  javascript,
  json,
  jsx,
  markdown,
  markup, // covers html / xml / svg
  python,
  rust,
  toml,
  tsx,
  typescript,
  yaml,
];

export function configurePrism(ctx: Ctx): void {
  ctx.set(prismConfig.key, {
    configureRefractor: (refractor) => {
      for (const lang of LANGUAGES) {
        refractor.register(lang);
      }
      // Aliases users actually type in fenced blocks.
      refractor.alias({
        bash: ["sh", "shell", "zsh"],
        markup: ["html", "xml"],
        javascript: ["js"],
        typescript: ["ts"],
      });
      return refractor;
    },
  });
}
