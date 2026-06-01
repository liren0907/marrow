// Mock implementations of every command exported from $lib/workspace/tauri.
//
// Match each wrapper's signature 1:1 so tauri.ts can route to either the
// real `invoke()` or the matching function here without any per-callsite
// type juggling. Order in this file mirrors order in tauri.ts for sanity.
//
// Coverage tiers:
//   - File CRUD + workspace open: real implementations against MockFs
//   - Recent workspaces / git / watcher status / app config: realistic
//     enough that the boot path runs (auto-reopen finds "Demo Workspace")
//   - Search / snapshots / conversion / graph layout: safe stubs that
//     return empty / null / no-op so consumers don't crash
//
// All functions are async to match the real wrappers — callers await them.

import { mockFs } from "./fs";
import {
  DEMO_WORKSPACE_NAME,
  DEMO_WORKSPACE_ROOT,
  ensureDemoSeeded,
} from "./demoData";
import type {
  DirEntry,
  FileMeta,
  ReadResult,
  WorkspaceInfo,
  WriteResult,
} from "$lib/workspace/types";
import { classifyFile } from "$lib/workspace/fileKind";

function basename(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx >= 0 ? path.slice(idx + 1) : path;
}

// ─── Dialog ─────────────────────────────────────────────────────────────
// Real impl pops a system folder picker. In browser mode there's no
// equivalent without File System Access API (Chrome-only and prompts the
// user every time), so for MVP we just hand back the demo workspace path.
// The user effectively can't "browse" — they get the seeded workspace.
export async function openDirectoryDialog(): Promise<string | null> {
  ensureDemoSeeded();
  return DEMO_WORKSPACE_ROOT;
}

// ─── Workspace lifecycle ────────────────────────────────────────────────
export async function openWorkspaceCmd(path: string): Promise<WorkspaceInfo> {
  ensureDemoSeeded();
  // Accept any path — in browser mode there's only ever one workspace.
  // Returning name=DEMO regardless keeps the title bar consistent.
  return { root: path, name: DEMO_WORKSPACE_NAME };
}

export async function listDirectory(path: string): Promise<DirEntry[]> {
  ensureDemoSeeded();
  return mockFs.listDir(path).map((f) => ({
    name: basename(f.path),
    path: f.path,
    is_dir: f.isDir,
    size: f.size,
    mtime: f.mtime,
  }));
}

export async function listWorkspaceFiles(root: string): Promise<FileMeta[]> {
  ensureDemoSeeded();
  return mockFs.walk(root).map((f) => ({
    path: f.path,
    name: basename(f.path),
    kind: classifyFile(f.path),
  }));
}

// ─── Search ─────────────────────────────────────────────────────────────
// Stubbed empty for MVP. Phase 2 swaps to in-memory regex over .md content.
export interface SearchHit {
  path: string;
  line: number;
  content: string;
  match_start: number;
  match_end: number;
}

export interface CrossHit {
  workspace_id: string;
  workspace_name: string;
  workspace_root: string;
  hit: SearchHit;
}

export async function searchWorkspace(
  _root: string,
  _query: string,
  _maxResults?: number,
): Promise<SearchHit[]> {
  return [];
}

export async function searchAllWorkspaces(
  _query: string,
  _maxResults?: number,
): Promise<CrossHit[]> {
  return [];
}

// ─── File IO ────────────────────────────────────────────────────────────
export async function readTextFile(path: string): Promise<ReadResult> {
  ensureDemoSeeded();
  const f = mockFs.get(path);
  if (!f || f.isDir) throw new Error(`Not found: ${path}`);
  return { content: f.content, mtime: f.mtime };
}

export async function readBinaryFile(path: string): Promise<Uint8Array> {
  ensureDemoSeeded();
  const f = mockFs.get(path);
  if (!f || f.isDir) throw new Error(`Not found: ${path}`);
  return f.bytes ?? new Uint8Array(0);
}

export async function writeTextFile(
  path: string,
  contents: string,
  expectedMtime?: number,
): Promise<WriteResult> {
  ensureDemoSeeded();
  const mtime = mockFs.writeText(path, contents, expectedMtime);
  return { mtime };
}

export async function writeBinaryFile(path: string, bytes: Uint8Array): Promise<void> {
  ensureDemoSeeded();
  mockFs.writeBinary(path, bytes);
}

export async function createFile(path: string): Promise<void> {
  ensureDemoSeeded();
  mockFs.createFile(path);
}

export async function createDirectory(path: string): Promise<void> {
  ensureDemoSeeded();
  mockFs.createDir(path);
}

export async function deletePath(path: string): Promise<void> {
  ensureDemoSeeded();
  mockFs.delete(path);
}

export async function renamePath(from: string, to: string): Promise<void> {
  ensureDemoSeeded();
  mockFs.rename(from, to);
}

// ─── Snapshots / file history ───────────────────────────────────────────
// No history in browser mode — return empty list so the history modal
// renders an "empty" state instead of throwing.
export interface SnapshotMeta {
  ts: number;
  hash: string;
  op: "save" | "rename" | "restore";
  size: number;
  prev_path: string | null;
}

export async function listFileHistory(_path: string): Promise<SnapshotMeta[]> {
  return [];
}

export async function readSnapshot(_hash: string): Promise<string> {
  throw new Error("Snapshots are not available in browser mode");
}

export async function restoreSnapshot(_path: string, _hash: string): Promise<void> {
  throw new Error("Snapshots are not available in browser mode");
}

// ─── Graph layout persistence ───────────────────────────────────────────
// Round-trips through localStorage so the user's layout survives reloads
// even in mock mode. Cheap, harmless, and graph view wants this.
const GRAPH_KEY = "marrow:devmock:graphLayout";

export async function loadGraphLayout(): Promise<Record<string, unknown> | null> {
  try {
    const raw = typeof localStorage !== "undefined" ? localStorage.getItem(GRAPH_KEY) : null;
    return raw ? (JSON.parse(raw) as Record<string, unknown>) : null;
  } catch {
    return null;
  }
}

export async function saveGraphLayout(data: Record<string, unknown>): Promise<void> {
  try {
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(GRAPH_KEY, JSON.stringify(data));
    }
  } catch {
    /* quota / serialisation failures are non-fatal */
  }
}

// ─── Recent workspaces ──────────────────────────────────────────────────
// We always advertise exactly one "recent" entry: the demo workspace.
// This makes the layout's `autoReopenRecent` automatically open the demo
// on first paint, so the user lands on the editor instead of the
// drop-folder empty state.
export interface WorkspaceSummary {
  id: string;
  name: string;
  last_path: string;
  last_opened_ts: number;
  created_ts: number;
}

export async function listRecentWorkspaces(_limit = 10): Promise<WorkspaceSummary[]> {
  ensureDemoSeeded();
  const ts = Date.parse("2025-04-28T10:00:00Z");
  return [
    {
      id: "demo",
      name: DEMO_WORKSPACE_NAME,
      last_path: DEMO_WORKSPACE_ROOT,
      last_opened_ts: ts,
      created_ts: ts,
    },
  ];
}

export async function forgetWorkspace(_id: string): Promise<void> {
  // No-op — the demo workspace is always there.
}

export async function pathExists(path: string): Promise<boolean> {
  ensureDemoSeeded();
  return mockFs.exists(path);
}

// ─── Git / watcher / app config ─────────────────────────────────────────
export async function getGitBranch(_root: string): Promise<string | null> {
  return null;
}

export interface WatcherStatus {
  running: boolean;
}

export async function getWatcherStatus(): Promise<WatcherStatus> {
  // Reporting "not running" is honest — there's no fs watcher in browser
  // mode. The status bar's watcher indicator should render as "off"
  // accordingly.
  return { running: false };
}

export async function setAppConfig(_args: {
  denyList: string[];
  watchDebounceMs: number;
  ownWriteTtlMs: number;
}): Promise<void> {
  // No backend to push config to — accept and discard.
}

// ─── Format conversion ──────────────────────────────────────────────────
// Real backend uses pandoc / native libs. In browser mode we have no
// converter, so fail loudly with a useful message instead of silently
// returning empty markdown that would look like a successful conversion.
export interface ConvertAsset {
  name: string;
  bytes_b64: string;
}

export interface ConvertResult {
  markdown: string;
  assets: ConvertAsset[];
}

const CONVERT_NOT_SUPPORTED = "Format conversion is not available in browser mode";

export async function convertToMarkdown(_path: string): Promise<string> {
  throw new Error(CONVERT_NOT_SUPPORTED);
}

export async function convertHtmlToMarkdown(_path: string): Promise<string> {
  throw new Error(CONVERT_NOT_SUPPORTED);
}

export async function convertPptxToMarkdown(_path: string): Promise<ConvertResult> {
  throw new Error(CONVERT_NOT_SUPPORTED);
}

export async function convertDocxToMarkdown(_path: string): Promise<ConvertResult> {
  throw new Error(CONVERT_NOT_SUPPORTED);
}

export async function extractAnnotations(
  _path: string,
): Promise<{ text: string; pos: string; count: number; spans: [number, number][] }[]> {
  // Browser dev mode has no NLP backend — return a small fixed sample so the
  // Distill view renders something instead of erroring.
  return [
    { text: "範例名詞", pos: "n", count: 3, spans: [] },
    { text: "知識圖譜", pos: "n", count: 2, spans: [] },
    { text: "marrow", pos: "eng", count: 1, spans: [] },
  ];
}
