use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::core::app_config::SharedAppConfig;

#[derive(Debug, Serialize, Clone)]
pub struct FsEventPayload {
    pub kind: &'static str,
    pub paths: Vec<String>,
}

pub struct WatcherState {
    watcher: Mutex<Option<RecommendedWatcher>>,
    event_tx: Mutex<Option<Sender<RawEvent>>>,
    recent_writes: Mutex<HashMap<PathBuf, Instant>>,
    /// Live config — debounce window and own-write TTL are read from
    /// here on every operation, so updates from the Settings page take
    /// effect on the next tick without rebuilding the watcher.
    app_config: SharedAppConfig,
}

#[derive(Debug)]
struct RawEvent {
    kind: EventKind,
    paths: Vec<PathBuf>,
}

fn canonicalize(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn classify(kind: EventKind) -> Option<&'static str> {
    match kind {
        EventKind::Create(_) => Some("create"),
        EventKind::Modify(notify::event::ModifyKind::Name(_)) => Some("rename"),
        EventKind::Modify(_) => Some("modify"),
        EventKind::Remove(_) => Some("remove"),
        _ => None,
    }
}

/// Read the current own-write TTL from shared config. Falls back to
/// 500ms if the lock is poisoned (should never happen in practice).
fn own_write_ttl(cfg: &SharedAppConfig) -> Duration {
    let ms = cfg
        .read()
        .map(|g| g.own_write_ttl_ms)
        .unwrap_or(500);
    Duration::from_millis(ms)
}

/// Read the current debounce window from shared config. Falls back to
/// 150ms if the lock is poisoned.
fn debounce_window(cfg: &SharedAppConfig) -> Duration {
    let ms = cfg
        .read()
        .map(|g| g.watch_debounce_ms)
        .unwrap_or(150);
    Duration::from_millis(ms)
}

impl WatcherState {
    pub fn new(app_config: SharedAppConfig) -> Self {
        Self {
            watcher: Mutex::new(None),
            event_tx: Mutex::new(None),
            recent_writes: Mutex::new(HashMap::new()),
            app_config,
        }
    }

    pub fn is_running(&self) -> bool {
        self.watcher.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn note_own_write(&self, path: &Path) {
        let canonical = canonicalize(path);
        let ttl = own_write_ttl(&self.app_config);
        let mut map = self.recent_writes.lock().unwrap();
        map.insert(canonical, Instant::now());
        let cutoff = Instant::now() - ttl;
        map.retain(|_, ts| *ts > cutoff);
    }

    fn is_own_write(&self, path: &Path) -> bool {
        let canonical = canonicalize(path);
        let ttl = own_write_ttl(&self.app_config);
        let mut map = self.recent_writes.lock().unwrap();
        let cutoff = Instant::now() - ttl;
        map.retain(|_, ts| *ts > cutoff);
        map.contains_key(&canonical)
    }

    pub fn start(&self, app: &AppHandle, root: &Path) -> Result<(), String> {
        // Drop any existing watcher & debouncer channel first.
        {
            let mut w = self.watcher.lock().unwrap();
            *w = None;
        }
        {
            let mut tx = self.event_tx.lock().unwrap();
            *tx = None;
        }

        let (tx, rx) = mpsc::channel::<RawEvent>();
        let tx_for_cb = tx.clone();

        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            if let Ok(Event { kind, paths, .. }) = res {
                let _ = tx_for_cb.send(RawEvent { kind, paths });
            }
        })
        .map_err(|e| format!("failed to create watcher: {e}"))?;

        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|e| format!("failed to watch {}: {e}", root.display()))?;

        *self.watcher.lock().unwrap() = Some(watcher);
        *self.event_tx.lock().unwrap() = Some(tx);

        // Debouncer thread: collect bursts, filter own-writes, emit.
        let app_handle = app.clone();
        // Clone the Arc for the worker thread so it can read live
        // debounce values without going through the AppHandle.
        let cfg = self.app_config.clone();
        thread::spawn(move || {
            let mut pending: HashMap<(&'static str, PathBuf), ()> = HashMap::new();
            let mut last_event_at: Option<Instant> = None;
            loop {
                let recv_timeout = if last_event_at.is_some() {
                    Duration::from_millis(30)
                } else {
                    Duration::from_millis(500)
                };
                match rx.recv_timeout(recv_timeout) {
                    Ok(raw) => {
                        let Some(kind_str) = classify(raw.kind) else { continue };
                        for p in raw.paths {
                            pending.insert((kind_str, canonicalize(&p)), ());
                        }
                        last_event_at = Some(Instant::now());
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(start) = last_event_at {
                            // Re-read the debounce window each tick so
                            // the user can tighten/loosen it from the
                            // Settings page without restarting.
                            if start.elapsed() >= debounce_window(&cfg)
                                && !pending.is_empty()
                            {
                                flush(&app_handle, &mut pending);
                                last_event_at = None;
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        if !pending.is_empty() {
                            flush(&app_handle, &mut pending);
                        }
                        return;
                    }
                }
            }
        });

        Ok(())
    }
}

fn flush(app: &AppHandle, pending: &mut HashMap<(&'static str, PathBuf), ()>) {
    // Bucket by kind, filter own-writes using managed state.
    let state = app.try_state::<WatcherState>();
    let mut buckets: HashMap<&'static str, Vec<String>> = HashMap::new();
    for ((kind, path), _) in pending.drain() {
        if let Some(ref s) = state {
            if s.is_own_write(&path) {
                continue;
            }
        }
        buckets
            .entry(kind)
            .or_default()
            .push(path.to_string_lossy().to_string());
    }
    for (kind, paths) in buckets {
        if paths.is_empty() {
            continue;
        }
        let _ = app.emit("fs-event", FsEventPayload { kind, paths });
    }
}
