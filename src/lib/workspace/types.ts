export type FileKind =
  | "markdown"
  | "image"
  | "video"
  | "audio"
  | "text"
  | "pdf"
  | "excalidraw"
  | "graph"
  | "convert"
  | "settings"
  | "unsupported";

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  mtime: number;
}

export interface WorkspaceInfo {
  root: string;
  name: string;
}

export interface WriteResult {
  mtime: number;
}

export interface ReadResult {
  content: string;
  mtime: number;
}

export type FsEventKind = "create" | "modify" | "remove" | "rename";

export interface FsEventPayload {
  kind: FsEventKind;
  paths: string[];
}

export interface FileMeta {
  path: string;
  name: string;
  kind: FileKind;
}

export interface Tab {
  id: string;
  path: string;
  kind: FileKind;
  title: string;
  isDirty: boolean;
  lastKnownMtime?: number;
  lastSavedTs?: number;
  reloadToken?: number;
  missing?: boolean;
  // Markdown tabs only — toggles between Milkdown WYSIWYG ("pretty") and a
  // CodeMirror plain-text source view ("raw"). Optional for back-compat;
  // readers default to "pretty" when undefined.
  viewMode?: "pretty" | "raw";
}

export interface Pane {
  id: string;
  tabs: Tab[];
  activeTabId: string | null;
}
