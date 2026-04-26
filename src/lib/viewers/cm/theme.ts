import type { Extension } from "@codemirror/state";
import { appearance } from "$lib/settings/appearanceSettings.svelte";
import { resolveCmTheme } from "./themes";

// Resolves the active CodeMirror theme for the current chrome theme,
// honoring the user's Settings → Appearance choice. The Settings page
// store value is read at call time, so reconfiguring the Compartment
// from a $effect picks up changes without a tab remount.
export function themeFor(daisyTheme: string): Extension | [] {
  return resolveCmTheme(appearance.cmTheme, daisyTheme);
}
