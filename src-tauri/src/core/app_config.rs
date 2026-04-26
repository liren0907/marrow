// Tunable knobs that originate in the frontend Settings UI and are
// pushed into Rust via `set_app_config` on app startup + on every
// change. Rust owns no persistence — the localStorage on the
// frontend is the single source of truth.
//
// Design:
//   * `AppConfig` is the plain data struct.
//   * `SharedAppConfig` is `Arc<RwLock<AppConfig>>` — a clone is
//     handed to every consumer that needs to read it (WatcherState
//     holds one for live debounce/TTL, walkers snapshot deny_list at
//     call entry, etc.). RwLock means many concurrent readers + one
//     writer; reads in hot loops are essentially free.
//   * `AppConfigState` is the Tauri-managed wrapper that the frontend
//     command writes through. It just owns the same Arc.
//
// On boot (before frontend mounts) Rust uses `Default::default()`,
// which mirrors the frontend's defaults exactly. The first push from
// the frontend therefore is usually a no-op.

use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Folder basenames to skip when walking the workspace tree
    /// (file index + full-text search). Matched as exact basename.
    pub deny_list: Vec<String>,
    /// Filesystem watcher quiet-period before flushing a batch.
    pub watch_debounce_ms: u64,
    /// Time-to-live for the "we just wrote this ourselves" set, used
    /// to filter out fs-events caused by Marrow's own saves.
    pub own_write_ttl_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            deny_list: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                ".obsidian".to_string(),
                ".marrow".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".svelte-kit".to_string(),
                ".next".to_string(),
                ".cache".to_string(),
            ],
            watch_debounce_ms: 150,
            own_write_ttl_ms: 500,
        }
    }
}

pub type SharedAppConfig = Arc<RwLock<AppConfig>>;

pub fn new_shared() -> SharedAppConfig {
    Arc::new(RwLock::new(AppConfig::default()))
}

/// Tauri-managed handle. Frontend writes through this via the
/// `set_app_config` command.
pub struct AppConfigState(pub SharedAppConfig);

impl AppConfigState {
    pub fn snapshot(&self) -> AppConfig {
        self.0.read().map(|g| g.clone()).unwrap_or_default()
    }
}

/// Replace the live config. Validation (basename charset, sane
/// numeric ranges) happens on the frontend; Rust still clamps on
/// receive as a defense-in-depth measure so a buggy/forged caller
/// can't tank the watcher.
pub fn apply_config(
    shared: &SharedAppConfig,
    deny_list: Vec<String>,
    watch_debounce_ms: u64,
    own_write_ttl_ms: u64,
) {
    let cleaned_deny: Vec<String> = deny_list
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.contains('/') && !s.contains('\\') && s != "..")
        .take(200) // hard cap so a forged caller can't bloat the Vec
        .collect();
    let watch = watch_debounce_ms.clamp(50, 1000);
    let ttl = own_write_ttl_ms.clamp(100, 2000);
    if let Ok(mut guard) = shared.write() {
        guard.deny_list = cleaned_deny;
        guard.watch_debounce_ms = watch;
        guard.own_write_ttl_ms = ttl;
    }
}

#[tauri::command]
pub fn set_app_config(
    state: tauri::State<'_, AppConfigState>,
    deny_list: Vec<String>,
    watch_debounce_ms: u64,
    own_write_ttl_ms: u64,
) -> Result<(), String> {
    apply_config(
        &state.0,
        deny_list,
        watch_debounce_ms,
        own_write_ttl_ms,
    );
    Ok(())
}
