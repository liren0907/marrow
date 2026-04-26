import { $prose } from "@milkdown/utils";
import { Plugin, PluginKey } from "@milkdown/prose/state";
import type { EditorView } from "@milkdown/prose/view";
import { workspace } from "$lib/workspace/workspace.svelte";
import { writeBinaryFile } from "$lib/workspace/tauri";
import { workspaceSettings } from "$lib/settings/workspaceSettings.svelte";
import { advancedSettings } from "$lib/settings/advancedSettings.svelte";
import { showError } from "$lib/stores/toastStore.svelte";

function extFromMime(mime: string): string {
  switch (mime) {
    case "image/png":
      return "png";
    case "image/jpeg":
    case "image/jpg":
      return "jpg";
    case "image/gif":
      return "gif";
    case "image/webp":
      return "webp";
    case "image/bmp":
      return "bmp";
    case "image/svg+xml":
      return "svg";
    default:
      return "png";
  }
}

function timestampSlug(): string {
  const d = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");
  return (
    `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}` +
    `-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`
  );
}

function randomSuffix(): string {
  return Math.random().toString(36).slice(2, 8);
}

function joinPath(parent: string, child: string): string {
  const sep = parent.includes("\\") ? "\\" : "/";
  return parent.endsWith(sep) ? `${parent}${child}` : `${parent}${sep}${child}`;
}

async function handleImages(view: EditorView, files: File[]): Promise<void> {
  const root = workspace.info?.root;
  if (!root) {
    showError("Open a workspace before pasting images");
    return;
  }
  // Resolve the user-configured attachment folder + size cap once per
  // paste batch so toggling them mid-paste can't split files across
  // folders or apply different limits to siblings.
  const attachDir = workspaceSettings.attachmentFolder;
  const maxBytes = advancedSettings.imagePasteMaxBytes;
  const insertions: string[] = [];
  for (const file of files) {
    if (file.size > maxBytes) {
      showError(`Image too large (>${Math.round(maxBytes / 1024 / 1024)}MB), skipped`);
      continue;
    }
    try {
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      const ext = extFromMime(file.type);
      const filename = `pasted-${timestampSlug()}-${randomSuffix()}.${ext}`;
      const fullPath = joinPath(joinPath(root, attachDir), filename);
      await writeBinaryFile(fullPath, bytes);
      insertions.push(`![${filename}](${attachDir}/${filename})`);
    } catch (e) {
      showError(`Failed to write image: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
  if (insertions.length === 0) return;
  const text = insertions.join("\n\n");
  const tr = view.state.tr.insertText(text, view.state.selection.from);
  view.dispatch(tr);
}

export const imagePastePlugin = $prose(
  () =>
    new Plugin({
      key: new PluginKey("marrow-image-paste"),
      props: {
        handlePaste(view, event) {
          const items = event.clipboardData?.items;
          if (!items) return false;
          const images: File[] = [];
          for (const item of items) {
            if (item.kind === "file" && item.type.startsWith("image/")) {
              const file = item.getAsFile();
              if (file) images.push(file);
            }
          }
          if (images.length === 0) return false;
          event.preventDefault();
          void handleImages(view, images);
          return true;
        },
      },
    }),
);
