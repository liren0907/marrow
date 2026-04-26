import { invoke } from "@tauri-apps/api/core";
import type {
  DirEntry,
  FileMeta,
  ReadResult,
  WorkspaceInfo,
  WriteResult,
} from "./types";

export async function openDirectoryDialog(): Promise<string | null> {
  try {
    return await invoke<string>("open_directory_dialog");
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    if (msg === "No folder selected") return null;
    throw e;
  }
}

export function openWorkspaceCmd(path: string): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("open_workspace", { path });
}

export function listDirectory(path: string): Promise<DirEntry[]> {
  return invoke<DirEntry[]>("list_directory", { path });
}

export function listWorkspaceFiles(root: string): Promise<FileMeta[]> {
  return invoke<FileMeta[]>("list_workspace_files", { root });
}

export interface SearchHit {
  path: string;
  line: number;
  content: string;
  match_start: number;
  match_end: number;
}

export function searchWorkspace(
  root: string,
  query: string,
  maxResults?: number,
): Promise<SearchHit[]> {
  return invoke<SearchHit[]>("search_workspace", {
    root,
    query,
    maxResults: maxResults ?? null,
  });
}

export interface CrossHit {
  workspace_id: string;
  workspace_name: string;
  workspace_root: string;
  hit: SearchHit;
}

export function searchAllWorkspaces(
  query: string,
  maxResults?: number,
): Promise<CrossHit[]> {
  return invoke<CrossHit[]>("search_all_workspaces", {
    query,
    maxResults: maxResults ?? null,
  });
}

export function readTextFile(path: string): Promise<ReadResult> {
  return invoke<ReadResult>("read_text_file", { path });
}

export function readBinaryFile(path: string): Promise<Uint8Array> {
  return invoke<number[]>("read_binary_file", { path }).then(
    (arr) => new Uint8Array(arr),
  );
}

export function writeTextFile(
  path: string,
  contents: string,
  expectedMtime?: number,
): Promise<WriteResult> {
  return invoke<WriteResult>("write_text_file", {
    path,
    contents,
    expectedMtime: expectedMtime ?? null,
  });
}

export function writeBinaryFile(path: string, bytes: Uint8Array): Promise<void> {
  return invoke<void>("write_binary_file", { path, bytes: Array.from(bytes) });
}

export function createFile(path: string): Promise<void> {
  return invoke<void>("create_file", { path });
}

export function createDirectory(path: string): Promise<void> {
  return invoke<void>("create_directory", { path });
}

export function deletePath(path: string): Promise<void> {
  return invoke<void>("delete_path", { path });
}

export function renamePath(from: string, to: string): Promise<void> {
  return invoke<void>("rename_path", { from, to });
}

export interface SnapshotMeta {
  ts: number;
  hash: string;
  op: "save" | "rename" | "restore";
  size: number;
  prev_path: string | null;
}

export function listFileHistory(path: string): Promise<SnapshotMeta[]> {
  return invoke<SnapshotMeta[]>("list_file_history", { path });
}

export function readSnapshot(hash: string): Promise<string> {
  return invoke<string>("read_snapshot", { hash });
}

export function restoreSnapshot(path: string, hash: string): Promise<void> {
  return invoke<void>("restore_snapshot", { path, hash });
}

export function loadGraphLayout(): Promise<Record<string, unknown> | null> {
  return invoke<Record<string, unknown> | null>("load_graph_layout");
}

export function saveGraphLayout(data: Record<string, unknown>): Promise<void> {
  return invoke<void>("save_graph_layout", { data });
}

export interface WorkspaceSummary {
  id: string;
  name: string;
  last_path: string;
  last_opened_ts: number;
  created_ts: number;
}

export function listRecentWorkspaces(limit = 10): Promise<WorkspaceSummary[]> {
  return invoke<WorkspaceSummary[]>("list_recent_workspaces", { limit });
}

export function forgetWorkspace(id: string): Promise<void> {
  return invoke<void>("forget_workspace", { id });
}

export function pathExists(path: string): Promise<boolean> {
  return invoke<boolean>("path_exists", { path });
}

export function getGitBranch(root: string): Promise<string | null> {
  return invoke<string | null>("get_git_branch", { root });
}

export interface WatcherStatus {
  running: boolean;
}

export function getWatcherStatus(): Promise<WatcherStatus> {
  return invoke<WatcherStatus>("get_watcher_status");
}

export function convertToMarkdown(path: string): Promise<string> {
  return invoke<string>("convert_to_markdown", { path });
}

export function convertHtmlToMarkdown(path: string): Promise<string> {
  return invoke<string>("convert_html_to_markdown", { path });
}

export interface ConvertAsset {
  /** Final filename — caller writes to `attachments/<name>` on save. */
  name: string;
  /** Base64-encoded raw bytes. Decode before writing. */
  bytes_b64: string;
}

export interface ConvertResult {
  markdown: string;
  assets: ConvertAsset[];
}

export function convertPptxToMarkdown(path: string): Promise<ConvertResult> {
  return invoke<ConvertResult>("convert_pptx_to_markdown", { path });
}

export function convertDocxToMarkdown(path: string): Promise<ConvertResult> {
  return invoke<ConvertResult>("convert_docx_to_markdown", { path });
}

/** Push the user's app config knobs (deny list, watch debounce, own-write
 *  TTL) to the Rust backend. localStorage on the frontend is the source
 *  of truth — Rust holds a live copy that walkers + watcher read from. */
export function setAppConfig(args: {
  denyList: string[];
  watchDebounceMs: number;
  ownWriteTtlMs: number;
}): Promise<void> {
  return invoke<void>("set_app_config", {
    denyList: args.denyList,
    watchDebounceMs: args.watchDebounceMs,
    ownWriteTtlMs: args.ownWriteTtlMs,
  });
}
