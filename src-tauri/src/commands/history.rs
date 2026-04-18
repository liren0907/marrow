use std::fs;
use std::path::Path;
use tauri::State;

use crate::core::db::{DbState, SnapshotMeta};
use crate::core::fs_watch::WatcherState;

#[tauri::command]
pub async fn list_file_history(
    path: String,
    db: State<'_, DbState>,
) -> Result<Vec<SnapshotMeta>, String> {
    let ctx = db.current_ctx().await.ok_or("no workspace open")?;
    let rel = DbState::to_rel_path(&ctx, Path::new(&path))
        .ok_or_else(|| format!("path not inside workspace: {path}"))?;
    db.list_history(&ctx, &rel).await
}

#[tauri::command]
pub async fn read_snapshot(hash: String, db: State<'_, DbState>) -> Result<String, String> {
    let bytes = db.read_blob(&hash)?;
    String::from_utf8(bytes).map_err(|e| format!("blob is not utf-8: {e}"))
}

#[tauri::command]
pub async fn restore_snapshot(
    path: String,
    hash: String,
    db: State<'_, DbState>,
    watcher: State<'_, WatcherState>,
) -> Result<(), String> {
    let ctx = db.current_ctx().await.ok_or("no workspace open")?;
    let rel = DbState::to_rel_path(&ctx, Path::new(&path))
        .ok_or_else(|| format!("path not inside workspace: {path}"))?;

    // 1. Load historical content.
    let historical = db.read_blob(&hash)?;

    // 2. Snapshot current file content BEFORE overwriting so restore is reversible.
    if let Ok(current) = fs::read(&path) {
        let _ = db.snapshot_save(&ctx, &rel, &current).await;
    }

    // 3. Write historical content to disk, suppressing fs-watch own-write echo.
    let p = Path::new(&path);
    watcher.note_own_write(p);
    fs::write(p, &historical).map_err(|e| format!("write: {e}"))?;

    // 4. Record the restore itself.
    let _ = db.snapshot_restore(&ctx, &rel, &historical).await;
    Ok(())
}
