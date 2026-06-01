// HTTP transport for the Distill page — talks to the dev-only Rust HTTP bridge
// (src-tauri/src/devserver.rs) on a fixed localhost port. Used when the app
// runs in a plain browser (no Tauri `invoke`), so the page can hit the REAL
// backend instead of the in-memory mock. The bridge only exists in debug
// builds, so this transport is effectively dev-only too.

import type { Annotation } from "./tauri";

const BASE = "http://localhost:7080";

/** L0 → L1 extraction over HTTP (mirrors the `extract_annotations` command). */
export async function extractAnnotationsHttp(path: string): Promise<Annotation[]> {
  const res = await fetch(`${BASE}/distill/annotations`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ path }),
  });
  if (!res.ok) {
    throw new Error(`devserver ${res.status}: ${await res.text()}`);
  }
  return res.json();
}

/** One `.md` file under a real folder, as listed by `GET /distill/tree`.
 *  Mirrors `TreeEntry` on the Rust side. */
export interface DistillTreeEntry {
  /** Absolute path — send this to `extractAnnotationsHttp`. */
  path: string;
  /** Path relative to the queried root — for display in the picker. */
  rel: string;
}

/** List the `.md` files under a real folder so the browser Distill page can
 *  pick a real source. Dev-HTTP-only — there is no Tauri/mock equivalent
 *  (the native app follows real panes instead). */
export async function listDistillTreeHttp(root: string): Promise<DistillTreeEntry[]> {
  const res = await fetch(`${BASE}/distill/tree?root=${encodeURIComponent(root)}`);
  if (!res.ok) {
    throw new Error(`devserver ${res.status}: ${await res.text()}`);
  }
  return res.json();
}
