use std::fs;
use std::path::Path;
use tauri::State;

use crate::core::db::{DbHandle, SnapshotMeta};
use crate::core::fs_watch::WatcherState;

#[tauri::command]
pub async fn list_file_history(
    path: String,
    db: State<'_, DbHandle>,
) -> Result<Vec<SnapshotMeta>, String> {
    let state = db.current().await.ok_or("no workspace open")?;
    let rel = state
        .to_rel_path(Path::new(&path))
        .ok_or_else(|| format!("path not inside workspace: {path}"))?;
    state.list_history(&rel).await
}

#[tauri::command]
pub async fn read_snapshot(hash: String, db: State<'_, DbHandle>) -> Result<String, String> {
    let state = db.current().await.ok_or("no workspace open")?;
    let bytes = state.read_blob(&hash)?;
    String::from_utf8(bytes).map_err(|e| format!("blob is not utf-8: {e}"))
}

#[tauri::command]
pub async fn restore_snapshot(
    path: String,
    hash: String,
    db: State<'_, DbHandle>,
    watcher: State<'_, WatcherState>,
) -> Result<(), String> {
    let state = db.current().await.ok_or("no workspace open")?;
    let rel = state
        .to_rel_path(Path::new(&path))
        .ok_or_else(|| format!("path not inside workspace: {path}"))?;

    // 1. Load historical content.
    let historical = state.read_blob(&hash)?;

    // 2. Snapshot current file content BEFORE overwriting so restore is reversible.
    if let Ok(current) = fs::read(&path) {
        let _ = state.snapshot_save(&rel, &current).await;
    }

    // 3. Write historical content to disk, suppressing fs-watch own-write echo.
    let p = Path::new(&path);
    watcher.note_own_write(p);
    fs::write(p, &historical).map_err(|e| format!("write: {e}"))?;

    // 4. Record the restore itself.
    let _ = state.snapshot_restore(&rel, &historical).await;
    Ok(())
}
