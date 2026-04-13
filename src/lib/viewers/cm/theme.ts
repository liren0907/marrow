import type { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";

export function themeFor(daisyTheme: string): Extension | [] {
  const isDark = daisyTheme === "dark" || daisyTheme === "synthwave";
  return isDark ? oneDark : [];
}
