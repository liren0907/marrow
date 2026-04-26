// Swaps the active Prism stylesheet by toggling a single <link> tag in
// the document head. Vite resolves `?url` imports at build time so the
// CSS files get hashed + included in the bundle — no runtime fetch.
//
// Called once from +layout.svelte on mount (`initPrismTheme`), and
// again whenever the user changes the setting (an effect re-applies).

import {
  registerPrismApplyHook,
  type PrismThemeKey,
} from "./appearanceSettings.svelte";

import defaultUrl from "prismjs/themes/prism.css?url";
import okaidiaUrl from "prismjs/themes/prism-okaidia.css?url";
import tomorrowUrl from "prismjs/themes/prism-tomorrow.css?url";
import solarizedLightUrl from "prismjs/themes/prism-solarizedlight.css?url";

const URL_BY_KEY: Record<PrismThemeKey, string> = {
  "default": defaultUrl,
  "okaidia": okaidiaUrl,
  "tomorrow": tomorrowUrl,
  "solarized-light": solarizedLightUrl,
};

const LINK_ID = "marrow-prism-theme";

function applyPrismTheme(key: PrismThemeKey): void {
  if (typeof document === "undefined") return;
  const href = URL_BY_KEY[key];
  if (!href) return;
  let el = document.getElementById(LINK_ID) as HTMLLinkElement | null;
  if (!el) {
    el = document.createElement("link");
    el.id = LINK_ID;
    el.rel = "stylesheet";
    document.head.appendChild(el);
  }
  if (el.href !== new URL(href, document.baseURI).href) {
    el.href = href;
  }
}

/** Wire the apply-hook so the Settings setter triggers a swap, and
 *  install the initial stylesheet for the currently persisted choice. */
export function initPrismTheme(): void {
  registerPrismApplyHook(applyPrismTheme);
}
