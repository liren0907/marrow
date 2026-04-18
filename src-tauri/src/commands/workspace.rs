use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, State};

use crate::core::db::DbState;
use crate::core::fs_watch::WatcherState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub root: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteResult {
    pub mtime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadResult {
    pub content: String,
    pub mtime: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: String,
    pub name: String,
    pub kind: &'static str,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchHit {
    pub path: String,
    pub line: u64,
    pub content: String,
    pub match_start: usize,
    pub match_end: usize,
}

const DENY_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".obsidian",
    ".marrow",
    "target",
    "dist",
    "build",
    ".svelte-kit",
    ".next",
    ".cache",
];

fn classify_ext(ext: &str) -> Option<&'static str> {
    let lower = ext.to_ascii_lowercase();
    match lower.as_str() {
        "md" | "markdown" | "mdx" => Some("markdown"),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "avif" => {
            Some("image")
        }
        "mp4" | "webm" | "mov" | "avi" | "mkv" | "m4v" => Some("video"),
        "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => Some("audio"),
        "pdf" => Some("pdf"),
        "txt" | "json" | "yaml" | "yml" | "toml" | "xml" | "csv" | "log" | "ini" | "conf"
        | "sh" | "bash" | "zsh" | "js" | "ts" | "jsx" | "tsx" | "mjs" | "cjs" | "py" | "rb"
        | "go" | "rs" | "c" | "cpp" | "h" | "hpp" | "java" | "kt" | "swift" | "php" | "html"
        | "htm" | "css" | "scss" | "less" | "vue" | "svelte" => Some("text"),
        _ => None,
    }
}

fn mtime_ms(md: &std::fs::Metadata) -> u64 {
    md.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[tauri::command]
pub async fn open_workspace(
    path: String,
    app: AppHandle,
    state: State<'_, WatcherState>,
    db: State<'_, DbState>,
) -> Result<WorkspaceInfo, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !p.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    state.start(&app, p)?;
    if p.join(".marrow").join("history").exists() {
        eprintln!(
            "[marrow] pre-migration {}/.marrow/history/ found; safe to delete",
            path
        );
    }
    if let Err(e) = db.activate_workspace(p).await {
        eprintln!("[marrow] failed to activate workspace DB for {}: {}", path, e);
    }
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    Ok(WorkspaceInfo { root: path, name })
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let p = Path::new(&path);
    if !p.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    let mut entries: Vec<DirEntry> = Vec::new();
    for entry in fs::read_dir(p).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let md = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        entries.push(DirEntry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir: md.is_dir(),
            size: md.len(),
            mtime: mtime_ms(&md),
        });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<ReadResult, String> {
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let md = fs::metadata(&path).map_err(|e| e.to_string())?;
    Ok(ReadResult {
        content,
        mtime: mtime_ms(&md),
    })
}

#[tauri::command]
pub fn read_binary_file(path: String) -> Result<Vec<u8>, String> {
    fs::read(&path).map_err(|e| format!("Failed to read {}: {}", path, e))
}

#[tauri::command]
pub fn list_workspace_files(root: String) -> Result<Vec<FileMeta>, String> {
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("Not a directory: {}", root));
    }
    let mut out: Vec<FileMeta> = Vec::new();

    let walker = ignore::WalkBuilder::new(root_path)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(false)
        .filter_entry(|e| {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                if DENY_DIRS.iter().any(|d| *d == name.as_ref()) {
                    return false;
                }
            }
            true
        })
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };
        if !file_type.is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(kind) = classify_ext(ext) else {
            continue;
        };
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(FileMeta {
            path: path.to_string_lossy().to_string(),
            name,
            kind,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn search_workspace(
    root: String,
    query: String,
    max_results: Option<usize>,
) -> Result<Vec<SearchHit>, String> {
    use grep::matcher::Matcher;
    use grep::regex::RegexMatcherBuilder;
    use grep::searcher::{sinks, Searcher};

    let trimmed = query.trim();
    if trimmed.len() < 2 {
        return Ok(vec![]);
    }
    let limit = max_results.unwrap_or(200);

    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("Not a directory: {}", root));
    }

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(true)
        .fixed_strings(true)
        .build(trimmed)
        .map_err(|e| format!("regex: {}", e))?;

    let walker = ignore::WalkBuilder::new(root_path)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(false)
        .filter_entry(|e| {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                if DENY_DIRS.iter().any(|d| *d == name.as_ref()) {
                    return false;
                }
            }
            true
        })
        .build();

    let mut hits: Vec<SearchHit> = Vec::new();
    let mut searcher = Searcher::new();

    'outer: for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };
        if !file_type.is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let kind = classify_ext(ext);
        if !matches!(kind, Some("markdown") | Some("text")) {
            continue;
        }

        if hits.len() >= limit {
            break 'outer;
        }

        let path_str = path.to_string_lossy().to_string();
        let remaining = limit.saturating_sub(hits.len());
        let mut file_hits: Vec<SearchHit> = Vec::new();

        let _ = searcher.search_path(
            &matcher,
            path,
            sinks::UTF8(|line_num, line_text| {
                if file_hits.len() >= remaining {
                    return Ok(false);
                }
                let trimmed_line = line_text.trim_end_matches(['\r', '\n']);
                let (start, end) = match matcher.find(trimmed_line.as_bytes()) {
                    Ok(Some(m)) => (m.start(), m.end()),
                    _ => (0, 0),
                };
                file_hits.push(SearchHit {
                    path: path_str.clone(),
                    line: line_num,
                    content: trimmed_line.to_string(),
                    match_start: start,
                    match_end: end,
                });
                Ok(true)
            }),
        );

        hits.extend(file_hits);
    }

    Ok(hits)
}

#[tauri::command]
pub fn create_file(
    path: String,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        return Err(format!("Already exists: {}", path));
    }
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("create parent dir: {}", e))?;
        }
    }
    state.note_own_write(p);
    fs::write(p, "").map_err(|e| format!("create file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn create_directory(
    path: String,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        return Err(format!("Already exists: {}", path));
    }
    state.note_own_write(p);
    fs::create_dir_all(p).map_err(|e| format!("create dir: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn delete_path(
    path: String,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Does not exist: {}", path));
    }
    // Refuse to delete a path that looks like a workspace root (no parent or
    // very shallow). The frontend should never request this, but we double-check.
    let canonical = p.canonicalize().map_err(|e| format!("canonicalize: {}", e))?;
    if canonical.parent().is_none() || canonical.components().count() <= 2 {
        return Err(format!("Refusing to delete root-like path: {}", path));
    }
    state.note_own_write(p);
    if p.is_dir() {
        fs::remove_dir_all(p).map_err(|e| format!("delete dir: {}", e))?;
    } else {
        fs::remove_file(p).map_err(|e| format!("delete file: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn rename_path(
    from: String,
    to: String,
    state: State<'_, WatcherState>,
    db: State<'_, DbState>,
) -> Result<(), String> {
    if from == to {
        return Err("Source and destination are identical".into());
    }
    let src = Path::new(&from);
    let dst = Path::new(&to);
    if !src.exists() {
        return Err(format!("Source does not exist: {}", from));
    }
    if dst.exists() {
        return Err(format!("Destination already exists: {}", to));
    }
    // The watcher will see remove(src) + create(dst); note both paths.
    state.note_own_write(src);
    state.note_own_write(dst);
    fs::rename(src, dst).map_err(|e| format!("rename: {}", e))?;
    // Record the rename in history DB (only for markdown files).
    if is_markdown(src) {
        if let Some(ctx) = db.current_ctx().await {
            if let (Some(from_rel), Some(to_rel)) = (
                DbState::to_rel_path(&ctx, src),
                DbState::to_rel_path(&ctx, dst),
            ) {
                if let Err(e) = db.snapshot_rename(&ctx, &from_rel, &to_rel).await {
                    eprintln!("[marrow] snapshot_rename failed: {}", e);
                }
            }
        }
    }
    Ok(())
}

fn is_markdown(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "md" | "markdown" | "mdx"))
        .unwrap_or(false)
}

#[tauri::command]
pub fn write_binary_file(
    path: String,
    bytes: Vec<u8>,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let p = Path::new(&path);
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("create parent dir: {}", e))?;
        }
    }
    state.note_own_write(p);
    fs::write(p, bytes).map_err(|e| format!("write binary: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn write_text_file(
    path: String,
    contents: String,
    expected_mtime: Option<u64>,
    state: State<'_, WatcherState>,
    db: State<'_, DbState>,
) -> Result<WriteResult, String> {
    if let Some(expected) = expected_mtime {
        if let Ok(md) = fs::metadata(&path) {
            let actual = mtime_ms(&md);
            // 1ms slack absorbs filesystem mtime rounding.
            if actual > expected + 1 {
                return Err(format!(
                    "File changed on disk (expected mtime {}, actual {})",
                    expected, actual
                ));
            }
        }
    }
    let p = Path::new(&path);
    state.note_own_write(p);
    fs::write(&path, &contents).map_err(|e| format!("Failed to write {}: {}", path, e))?;
    let md = fs::metadata(&path).map_err(|e| e.to_string())?;
    // Snapshot markdown saves into the history DB. Bounded by timeout so a
    // stuck DB can never block saves. Errors logged but never surface to UI.
    if is_markdown(p) {
        if let Some(ctx) = db.current_ctx().await {
            if let Some(rel) = DbState::to_rel_path(&ctx, p) {
                let bytes = contents.as_bytes().to_vec();
                let _ = tokio::time::timeout(
                    std::time::Duration::from_millis(1500),
                    async { db.snapshot_save(&ctx, &rel, &bytes).await },
                )
                .await;
            }
        }
    }
    Ok(WriteResult {
        mtime: mtime_ms(&md),
    })
}
