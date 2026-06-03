// Dev-only fixtures for the /gallery "viewers" route. The file viewers take a
// `tab: Tab` and render its content; here we hand them throwaway tab objects
// pointing at tiny sample assets (or, for the store-coupled ones, the real tab
// the gallery seed already opened). Nothing here touches Tauri or mutates app
// state — it stays entirely within the gallery.
import type { Tab } from "$lib/workspace/types";
import { workspace } from "$lib/workspace/workspace.svelte";
import sampleVideoUrl from "./samples/sample.mp4?url";
import sampleAudioUrl from "./samples/sample.wav?url";

// Inline so ImageTab has a real image without shipping a binary. In the browser
// `safeConvertFileSrc` returns this data URI unchanged.
export const sampleImageUrl =
  "data:image/svg+xml;charset=utf-8," +
  encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="240" height="136">` +
      `<rect width="100%" height="100%" fill="#6c6f64"/>` +
      `<text x="50%" y="50%" fill="#fff" font-family="sans-serif" font-size="15" ` +
      `text-anchor="middle" dominant-baseline="middle">sample image</text></svg>`,
  );

// A throwaway Tab. These never live in workspace.panes — fine for the read-only
// media / unsupported viewers (no autosave, no patchTab dependency).
function fakeTab(path: string, kind: Tab["kind"], title: string): Tab {
  return { id: `gallery-${kind}`, path, kind, title, isDirty: false };
}

export const imageTab = (): Tab => fakeTab(sampleImageUrl, "image", "sample image");
export const videoTab = (): Tab => fakeTab(sampleVideoUrl, "video", "sample.mp4");
export const audioTab = (): Tab => fakeTab(sampleAudioUrl, "audio", "sample.wav");
export const unsupportedTab = (): Tab => fakeTab("/demo/archive.zip", "unsupported", "archive.zip");

// TextTab reads its content via readTextFile → devmock; point it at an existing
// mock .md so it has real content + CodeMirror highlighting. patchTab() is a
// harmless no-op here (this tab isn't in a pane; TextTab is read-only).
export function textTab(): Tab {
  const md =
    workspace.fileIndex.find((f) => f.path.endsWith("code-samples.md")) ??
    workspace.fileIndex.find((f) => f.kind === "markdown");
  return fakeTab(md?.path ?? "/demo/code-samples.md", "text", "code-samples.md");
}

// MarkdownTab is heavily store-coupled (autosave → patchTab, which silently
// no-ops for a tab outside any pane). The gallery seed has already opened a real
// markdown tab into a real pane, so reuse THAT object: patchTab can find it, so
// there's no spurious conflict. Returns null pre-seed (the seed gate guards us).
export function seededMarkdownTab(): Tab | null {
  return workspace.activePane?.tabs[0] ?? null;
}
