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
  reloadToken?: number;
  missing?: boolean;
}

export interface Pane {
  id: string;
  tabs: Tab[];
  activeTabId: string | null;
}
