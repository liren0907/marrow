// Typed wrappers for every Rust command Marrow exposes to the frontend.
//
// This module is the single IPC seam. Every consumer imports from here —
// no other file calls `invoke()` directly. That property is what lets the
// browser mock backend exist: when `isInTauri` is false (i.e. running
// under `vite dev` without Tauri), each wrapper transparently delegates
// to the matching function in `$lib/devmock/commands` instead of `invoke`.
//
// Because the mock matches every signature 1:1, downstream code is
// completely unaware which backend is responding.

import { invoke } from "@tauri-apps/api/core";
import { isInTauri } from "$lib/devmock/detect";
import * as mock from "$lib/devmock/commands";
import { extractAnnotationsHttp } from "./distillHttp";
import type {
  DirEntry,
  FileMeta,
  ReadResult,
  WorkspaceInfo,
  WriteResult,
} from "./types";

export async function openDirectoryDialog(): Promise<string | null> {
  if (!isInTauri) return mock.openDirectoryDialog();
  try {
    return await invoke<string>("open_directory_dialog");
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    if (msg === "No folder selected") return null;
    throw e;
  }
}

export function openWorkspaceCmd(path: string): Promise<WorkspaceInfo> {
  if (!isInTauri) return mock.openWorkspaceCmd(path);
  return invoke<WorkspaceInfo>("open_workspace", { path });
}

export function listDirectory(path: string): Promise<DirEntry[]> {
  if (!isInTauri) return mock.listDirectory(path);
  return invoke<DirEntry[]>("list_directory", { path });
}

export function listWorkspaceFiles(root: string): Promise<FileMeta[]> {
  if (!isInTauri) return mock.listWorkspaceFiles(root);
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
  if (!isInTauri) return mock.searchWorkspace(root, query, maxResults);
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
  if (!isInTauri) return mock.searchAllWorkspaces(query, maxResults);
  return invoke<CrossHit[]>("search_all_workspaces", {
    query,
    maxResults: maxResults ?? null,
  });
}

export function readTextFile(path: string): Promise<ReadResult> {
  if (!isInTauri) return mock.readTextFile(path);
  return invoke<ReadResult>("read_text_file", { path });
}

export function readBinaryFile(path: string): Promise<Uint8Array> {
  if (!isInTauri) return mock.readBinaryFile(path);
  return invoke<number[]>("read_binary_file", { path }).then(
    (arr) => new Uint8Array(arr),
  );
}

export function writeTextFile(
  path: string,
  contents: string,
  expectedMtime?: number,
): Promise<WriteResult> {
  if (!isInTauri) return mock.writeTextFile(path, contents, expectedMtime);
  return invoke<WriteResult>("write_text_file", {
    path,
    contents,
    expectedMtime: expectedMtime ?? null,
  });
}

export function writeBinaryFile(path: string, bytes: Uint8Array): Promise<void> {
  if (!isInTauri) return mock.writeBinaryFile(path, bytes);
  return invoke<void>("write_binary_file", { path, bytes: Array.from(bytes) });
}

export function createFile(path: string): Promise<void> {
  if (!isInTauri) return mock.createFile(path);
  return invoke<void>("create_file", { path });
}

export function createDirectory(path: string): Promise<void> {
  if (!isInTauri) return mock.createDirectory(path);
  return invoke<void>("create_directory", { path });
}

export function deletePath(path: string): Promise<void> {
  if (!isInTauri) return mock.deletePath(path);
  return invoke<void>("delete_path", { path });
}

export function renamePath(from: string, to: string): Promise<void> {
  if (!isInTauri) return mock.renamePath(from, to);
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
  if (!isInTauri) return mock.listFileHistory(path);
  return invoke<SnapshotMeta[]>("list_file_history", { path });
}

export function readSnapshot(hash: string): Promise<string> {
  if (!isInTauri) return mock.readSnapshot(hash);
  return invoke<string>("read_snapshot", { hash });
}

export function restoreSnapshot(path: string, hash: string): Promise<void> {
  if (!isInTauri) return mock.restoreSnapshot(path, hash);
  return invoke<void>("restore_snapshot", { path, hash });
}

export function loadGraphLayout(): Promise<Record<string, unknown> | null> {
  if (!isInTauri) return mock.loadGraphLayout();
  return invoke<Record<string, unknown> | null>("load_graph_layout");
}

export function saveGraphLayout(data: Record<string, unknown>): Promise<void> {
  if (!isInTauri) return mock.saveGraphLayout(data);
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
  if (!isInTauri) return mock.listRecentWorkspaces(limit);
  return invoke<WorkspaceSummary[]>("list_recent_workspaces", { limit });
}

export function forgetWorkspace(id: string): Promise<void> {
  if (!isInTauri) return mock.forgetWorkspace(id);
  return invoke<void>("forget_workspace", { id });
}

export function pathExists(path: string): Promise<boolean> {
  if (!isInTauri) return mock.pathExists(path);
  return invoke<boolean>("path_exists", { path });
}

export function getGitBranch(root: string): Promise<string | null> {
  if (!isInTauri) return mock.getGitBranch(root);
  return invoke<string | null>("get_git_branch", { root });
}

export interface WatcherStatus {
  running: boolean;
}

export function getWatcherStatus(): Promise<WatcherStatus> {
  if (!isInTauri) return mock.getWatcherStatus();
  return invoke<WatcherStatus>("get_watcher_status");
}

export function convertToMarkdown(path: string): Promise<string> {
  if (!isInTauri) return mock.convertToMarkdown(path);
  return invoke<string>("convert_to_markdown", { path });
}

/** One extracted L1 term (multi-layer notes — Distill view). Mirrors
 * `nlp::Annotation` on the Rust side. `spans` are char offsets into the
 * extracted plain text (forward-compat; unused by the current view). */
export interface Annotation {
  text: string;
  pos: string;
  count: number;
  spans: [number, number][];
}

/** Which backend the Distill page talks to.
 * - `invoke`: Tauri IPC (native app)
 * - `http`:   dev HTTP bridge on :7080 (real backend, reachable from a browser)
 * - `mock`:   in-memory fake data (offline / no backend)
 * Default: Tauri → invoke, browser → http. A localStorage override
 * (`marrow.distill.transport`) wins if present — the toggle hook; its UI is
 * deferred. Only the Distill page uses this; every other command is unchanged. */
export type DistillTransport = "invoke" | "http" | "mock";
const DISTILL_TRANSPORT_KEY = "marrow.distill.transport";

/** Effective Distill transport: explicit localStorage override wins, else the
 * environment default (Tauri → invoke, browser → http). */
export function distillTransport(): DistillTransport {
  if (typeof localStorage !== "undefined") {
    const o = localStorage.getItem(DISTILL_TRANSPORT_KEY);
    if (o === "invoke" || o === "http" || o === "mock") return o;
  }
  return isInTauri ? "invoke" : "http";
}

/** Persist an explicit Distill transport override (drives the dev-only chip in
 * DistillTab). Pass through the same values the resolver accepts. */
export function setDistillTransport(t: DistillTransport): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(DISTILL_TRANSPORT_KEY, t);
  } catch {
    // ignore — private mode / quota
  }
}

/** L0 → L1: extract noun-ish terms (with frequency) from a Markdown file. */
export function extractAnnotations(path: string): Promise<Annotation[]> {
  const transport = distillTransport();
  if (transport === "invoke") return invoke<Annotation[]>("extract_annotations", { path });
  if (transport === "mock") return mock.extractAnnotations(path);
  return extractAnnotationsHttp(path);
}

export function convertHtmlToMarkdown(path: string): Promise<string> {
  if (!isInTauri) return mock.convertHtmlToMarkdown(path);
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
  if (!isInTauri) return mock.convertPptxToMarkdown(path);
  return invoke<ConvertResult>("convert_pptx_to_markdown", { path });
}

export function convertDocxToMarkdown(path: string): Promise<ConvertResult> {
  if (!isInTauri) return mock.convertDocxToMarkdown(path);
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
  if (!isInTauri) return mock.setAppConfig(args);
  return invoke<void>("set_app_config", {
    denyList: args.denyList,
    watchDebounceMs: args.watchDebounceMs,
    ownWriteTtlMs: args.ownWriteTtlMs,
  });
}
