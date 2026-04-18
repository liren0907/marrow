use crate::core::fs_watch::WatcherState;
use serde::Serialize;
use std::process::Command;
use tauri::State;

#[derive(Serialize)]
pub struct WatcherStatus {
    pub running: bool,
}

#[tauri::command]
pub fn get_watcher_status(state: State<'_, WatcherState>) -> WatcherStatus {
    WatcherStatus {
        running: state.is_running(),
    }
}

/// Return the current branch name for the workspace root, or None if the
/// directory is not a git repo / detached HEAD / git isn't on PATH.
#[tauri::command]
pub fn get_git_branch(root: String) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if name.is_empty() || name == "HEAD" {
        return None;
    }
    Some(name)
}
