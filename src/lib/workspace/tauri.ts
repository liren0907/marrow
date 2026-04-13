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
