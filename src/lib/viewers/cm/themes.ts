// CodeMirror 6 theme registry. The Settings page picks one of these
// keys; TextTab reconfigures its theme Compartment via this resolver.
//
// "auto" preserves the legacy behavior — match the Marrow theme:
//   marrow-pro-dark / dark / synthwave  → oneDark
//   anything else                       → no theme (vanilla light)
//
// All other keys resolve to a fixed theme regardless of light/dark
// chrome, so a user who really wants Dracula in light mode can have it.

import type { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import {
  dracula,
  githubLight,
  solarizedLight,
} from "@uiw/codemirror-themes-all";
import type { CodeMirrorThemeKey } from "$lib/settings/appearanceSettings.svelte";

function isDarkChrome(daisyTheme: string): boolean {
  return (
    daisyTheme === "dark" ||
    daisyTheme === "synthwave" ||
    daisyTheme === "marrow-pro-dark"
  );
}

export function resolveCmTheme(
  key: CodeMirrorThemeKey,
  daisyTheme: string,
): Extension | [] {
  switch (key) {
    case "auto":
      return isDarkChrome(daisyTheme) ? oneDark : [];
    case "one-dark":
      return oneDark;
    case "dracula":
      return dracula;
    case "github-light":
      return githubLight;
    case "solarized-light":
      return solarizedLight;
    default:
      return [];
  }
}
