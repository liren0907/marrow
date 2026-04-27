// In-memory filesystem for the browser mock backend.
//
// Stores entries keyed by absolute POSIX-style path. Both files and folders
// have explicit entries — folders carry an empty content blob so listing
// can distinguish "exists as dir" vs "missing". mtime is just `Date.now()`
// at write time; it doubles as the optimistic-concurrency token that
// `writeTextFile` checks against `expectedMtime`.
//
// Path conventions:
//   - All paths use "/" separators (the demo workspace is rooted at "/demo").
//   - No trailing slashes on directory paths.
//   - Root of the workspace is just the directory entry itself.
//
// The mock FS has no notion of permissions, symlinks, hidden files, or
// the deny-list filtering done by Rust's `list_workspace_files`. The demo
// data simply doesn't include any such entries, so nothing to filter.

export interface MockFile {
  path: string;
  isDir: boolean;
  /** Plain text for .md / source files; empty string for binaries (use `bytes` instead). */
  content: string;
  /** Present only for binary files (images, PDFs). */
  bytes?: Uint8Array;
  mtime: number;
  /** Bytes for size reporting. For text we use UTF-8 byte length lazily. */
  size: number;
}

function parentOf(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx > 0 ? path.slice(0, idx) : "/";
}

function basename(path: string): string {
  const idx = path.lastIndexOf("/");
  return idx >= 0 ? path.slice(idx + 1) : path;
}

function utf8Bytes(s: string): number {
  // Reasonable approximation without instantiating TextEncoder per call;
  // good enough for the size field shown in DirEntry. ASCII = exact, multi-
  // byte = slightly under but never used for anything load-bearing.
  return new Blob([s]).size;
}

export class MockFs {
  private entries = new Map<string, MockFile>();

  /** Bulk seed — used once at module init to load the demo workspace. */
  seed(files: Array<Omit<MockFile, "size"> & { size?: number }>): void {
    const now = Date.now();
    for (const f of files) {
      const size = f.size ?? (f.bytes ? f.bytes.byteLength : utf8Bytes(f.content));
      this.entries.set(f.path, {
        ...f,
        mtime: f.mtime || now,
        size,
      });
      // Synthesize parent dir entries so list_directory works on intermediate
      // paths even if the seed didn't explicitly include them.
      this.ensureDir(parentOf(f.path));
    }
  }

  private ensureDir(path: string): void {
    if (path === "/" || path === "") return;
    if (this.entries.has(path)) return;
    this.entries.set(path, {
      path,
      isDir: true,
      content: "",
      mtime: Date.now(),
      size: 0,
    });
    this.ensureDir(parentOf(path));
  }

  exists(path: string): boolean {
    return this.entries.has(path);
  }

  get(path: string): MockFile | undefined {
    return this.entries.get(path);
  }

  /** Direct children of `dir` (one level only). */
  listDir(dir: string): MockFile[] {
    const out: MockFile[] = [];
    for (const f of this.entries.values()) {
      if (f.path === dir) continue;
      if (parentOf(f.path) === dir) out.push(f);
    }
    // Folders first, then files; within each, name-sorted.
    return out.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return basename(a.path).localeCompare(basename(b.path));
    });
  }

  /** Recursive walk, returns every file (not folder) under `root`. */
  walk(root: string): MockFile[] {
    const out: MockFile[] = [];
    const prefix = root.endsWith("/") ? root : root + "/";
    for (const f of this.entries.values()) {
      if (f.isDir) continue;
      if (f.path === root) continue;
      if (f.path.startsWith(prefix)) out.push(f);
    }
    return out;
  }

  writeText(path: string, content: string, expectedMtime?: number): number {
    const existing = this.entries.get(path);
    if (existing && expectedMtime != null && existing.mtime !== expectedMtime) {
      throw new Error("File changed on disk");
    }
    this.ensureDir(parentOf(path));
    const mtime = Date.now();
    this.entries.set(path, {
      path,
      isDir: false,
      content,
      mtime,
      size: utf8Bytes(content),
    });
    return mtime;
  }

  writeBinary(path: string, bytes: Uint8Array): number {
    this.ensureDir(parentOf(path));
    const mtime = Date.now();
    this.entries.set(path, {
      path,
      isDir: false,
      content: "",
      bytes,
      mtime,
      size: bytes.byteLength,
    });
    return mtime;
  }

  createFile(path: string): void {
    if (this.entries.has(path)) throw new Error(`File already exists: ${path}`);
    this.writeText(path, "");
  }

  createDir(path: string): void {
    if (this.entries.has(path)) throw new Error(`Path already exists: ${path}`);
    this.ensureDir(path);
  }

  delete(path: string): void {
    if (!this.entries.has(path)) throw new Error(`Not found: ${path}`);
    // If it's a directory, remove all children too — mirrors Rust's
    // delete_path semantics (recursive remove).
    const target = this.entries.get(path)!;
    if (target.isDir) {
      const prefix = path.endsWith("/") ? path : path + "/";
      for (const key of Array.from(this.entries.keys())) {
        if (key === path || key.startsWith(prefix)) this.entries.delete(key);
      }
    } else {
      this.entries.delete(path);
    }
  }

  rename(from: string, to: string): void {
    const src = this.entries.get(from);
    if (!src) throw new Error(`Source not found: ${from}`);
    if (this.entries.has(to)) throw new Error(`Target already exists: ${to}`);
    this.ensureDir(parentOf(to));

    if (src.isDir) {
      const prefix = from.endsWith("/") ? from : from + "/";
      const moves: Array<[string, MockFile]> = [];
      for (const [key, val] of this.entries.entries()) {
        if (key === from) continue;
        if (key.startsWith(prefix)) {
          moves.push([key, val]);
        }
      }
      this.entries.delete(from);
      this.entries.set(to, { ...src, path: to });
      for (const [key, val] of moves) {
        const newKey = to + key.slice(from.length);
        this.entries.delete(key);
        this.entries.set(newKey, { ...val, path: newKey });
      }
    } else {
      this.entries.delete(from);
      this.entries.set(to, { ...src, path: to });
    }
  }

  /** Debug helper for the eventual DevPanel. */
  snapshot(): MockFile[] {
    return Array.from(this.entries.values());
  }
}

export const mockFs = new MockFs();
