//! Dev-only HTTP bridge for the Distill page.
//!
//! The whole module is compiled behind `#[cfg(debug_assertions)]` (see the
//! `mod devserver;` declaration in lib.rs) — it exists ONLY in debug builds
//! (`yarn tauri dev`) and is never present in a `tauri build` release bundle.
//!
//! Why it exists: Tauri `invoke` only works inside the native webview, so the
//! Distill page can't be exercised with the real backend from a plain browser.
//! This server exposes the same extraction logic over HTTP on a fixed localhost
//! port, letting a browser (and browser automation) hit the real jieba pipeline.
//!
//! Security: bound to 127.0.0.1 only, CORS restricted to the Vite dev origin,
//! and only read-only Distill endpoints are exposed (list `.md`, read one
//! file's text, extract annotations) — no write/delete surface.

use std::sync::{Arc, OnceLock};

use axum::{
    Json, Router,
    extract::Query,
    http::{HeaderValue, Method, StatusCode, header},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tower_http::cors::CorsLayer;

use crate::commands::distill::run_extract;

/// Fixed dev port (chosen by the user; not 8080).
const ADDR: &str = "127.0.0.1:7080";

#[derive(Deserialize)]
struct ExtractReq {
    path: String,
}

/// Process-wide shutdown signal for the dev server. `serve` parks on
/// `notified()` for its whole life; `shutdown()` wakes it so axum closes the
/// listener before the process tears down (see `shutdown` for the why).
fn shutdown_signal() -> &'static Arc<Notify> {
    static SIG: OnceLock<Arc<Notify>> = OnceLock::new();
    SIG.get_or_init(|| Arc::new(Notify::new()))
}

/// Tell the running dev server to stop. Called from Tauri's `RunEvent::Exit`
/// so closing the app deterministically releases port 7080 instead of relying
/// on the OS to reclaim the socket on process death. Non-blocking: it only
/// notifies and returns, so it never hangs app exit — OS reclamation is the
/// backstop if the async close doesn't finish in time.
pub fn shutdown() {
    shutdown_signal().notify_waiters();
}

/// Spawn the dev server on Tauri's tokio runtime. Non-fatal: a bind failure
/// (e.g. port already in use) only logs — it must never crash the app.
pub fn spawn() {
    let sig = shutdown_signal().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = serve(sig).await {
            eprintln!("[devserver] not started: {e}");
        }
    });
}

async fn serve(sig: Arc<Notify>) -> Result<(), String> {
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:1620"
                .parse::<HeaderValue>()
                .expect("static origin"),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/distill/tree", get(tree))
        .route("/distill/file", get(file))
        .route("/distill/annotations", post(annotations))
        .layer(cors);

    let listener = match tokio::net::TcpListener::bind(ADDR).await {
        Ok(l) => l,
        // A stale `marrow` from a previous dev run is the usual culprit, and
        // it's a separate process this one can't signal — so make it loud and
        // actionable rather than a single line lost in the cargo/vite output.
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            eprintln!("\n[devserver] ⚠ port {ADDR} already in use — a previous `marrow` is probably still running.");
            eprintln!("[devserver]   Distill's HTTP transport won't reach the browser until you free it:");
            eprintln!("[devserver]   lsof -ti:7080 | xargs kill\n");
            return Err(format!("bind {ADDR}: {e}"));
        }
        Err(e) => return Err(format!("bind {ADDR}: {e}")),
    };
    println!("devserver listening on http://{ADDR}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { sig.notified().await })
        .await
        .map_err(|e| format!("serve: {e}"))
}

/// `POST /distill/annotations` — same extraction the Tauri command runs, over
/// HTTP. CPU/IO work goes on a blocking thread like the command does.
async fn annotations(
    Json(req): Json<ExtractReq>,
) -> Result<Json<Vec<nlp::Annotation>>, (StatusCode, String)> {
    let result = tokio::task::spawn_blocking(move || run_extract(&req.path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("join: {e}")))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(result))
}

/// Standard directory deny list — mirrors the workspace walker's defaults
/// (see `commands/workspace.rs` and CLAUDE.md). Hardcoded here so this dev
/// endpoint stays decoupled from the live `AppConfigState`.
const DENY_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".obsidian",
    "target",
    "dist",
    "build",
    ".svelte-kit",
    ".next",
    ".cache",
];

#[derive(Deserialize)]
struct TreeQuery {
    root: String,
}

#[derive(Serialize)]
struct TreeEntry {
    /// Absolute path — what the browser sends back to `/distill/annotations`.
    path: String,
    /// Path relative to `root` — for display in the picker.
    rel: String,
}

/// `GET /distill/tree?root=<abs>` — list the `.md` files under a real folder so
/// the browser Distill page can pick a real source to extract. In the browser
/// the editor pane is mock, so this is the only route to real files. A bad
/// `root` is a client error (400).
async fn tree(Query(q): Query<TreeQuery>) -> Result<Json<Vec<TreeEntry>>, (StatusCode, String)> {
    let entries = tokio::task::spawn_blocking(move || walk_md(&q.root))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("join: {e}")))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(entries))
}

/// Walk `root` for `.md` files, skipping hidden entries and the deny-list dirs.
fn walk_md(root: &str) -> Result<Vec<TreeEntry>, String> {
    let root_path = std::path::Path::new(root);
    if !root_path.is_dir() {
        return Err(format!("not a directory: {root}"));
    }
    let walker = ignore::WalkBuilder::new(root_path)
        .hidden(true)
        .git_ignore(true)
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

    let mut out: Vec<TreeEntry> = Vec::new();
    for entry in walker {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let rel = path
            .strip_prefix(root_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        out.push(TreeEntry {
            path: path.to_string_lossy().to_string(),
            rel,
        });
    }
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

#[derive(Deserialize)]
struct FileQuery {
    path: String,
}

#[derive(Serialize)]
struct FileResult {
    content: String,
    mtime: u64,
}

/// `GET /distill/file?path=<abs>` — read a single file's text so the browser
/// Distill view can open a REAL on-disk note in its editor pane (the browser's
/// normal read path is the in-memory mock, which has no real files). Mirrors
/// the `read_text_file` command's `{content, mtime}` shape. Read-only by
/// design — there is deliberately no write counterpart. A bad path is a client
/// error (400).
async fn file(Query(q): Query<FileQuery>) -> Result<Json<FileResult>, (StatusCode, String)> {
    let result = tokio::task::spawn_blocking(move || read_file(&q.path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("join: {e}")))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

fn read_file(path: &str) -> Result<FileResult, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let mtime = std::fs::metadata(path)
        .ok()
        .and_then(|md| md.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Ok(FileResult { content, mtime })
}
