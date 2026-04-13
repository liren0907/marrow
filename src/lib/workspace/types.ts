export type FileKind =
  | "markdown"
  | "image"
  | "video"
  | "audio"
  | "text"
  | "pdf"
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

export interface Tab {
  id: string;
  path: string;
  kind: FileKind;
  title: string;
  isDirty: boolean;
  draftMarkdown?: string;
  lastSavedHash?: string;
  lastKnownMtime?: number;
}

export interface Pane {
  id: string;
  tabs: Tab[];
  activeTabId: string | null;
}
