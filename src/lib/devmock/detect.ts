// Runtime detection of "are we inside Tauri's webview, or a plain browser?"
//
// We pick at module init (constant) so consumers don't pay per-call cost.
// Tauri 2 injects `__TAURI_INTERNALS__`; older 1.x and some plugin code
// still reference `__TAURI__`. Accept either so the same detect logic works
// across rev'd Tauri versions and matches `safeConvertFileSrc` in
// $lib/utils/tauriUtils.ts (which uses `__TAURI__`).
//
// `isInTauri === false` is the trigger that flips $lib/workspace/tauri.ts
// over to the in-memory mock backend so `vite dev` (without Tauri) can run
// the full app for visual / browser-driven testing.

const w = typeof window !== "undefined" ? (window as unknown as Record<string, unknown>) : null;

export const isInTauri =
  w !== null && ("__TAURI_INTERNALS__" in w || "__TAURI__" in w);

export const isInBrowserMock = !isInTauri && typeof window !== "undefined";
