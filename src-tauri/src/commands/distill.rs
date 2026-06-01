//! Distill view commands — L0 → L1 extraction (multi-layer notes).
//!
//! Thin Tauri wrapper over the pure `nlp` crate. CPU-bound work runs inside
//! `spawn_blocking` (same pattern as the document converters) so the IPC
//! runtime isn't starved on large notes.

/// Shared extraction logic: load a Markdown file and run the L1 NLP pipeline.
/// Synchronous + blocking (file read + jieba) — callers wrap it in
/// `spawn_blocking`. Reused by both the Tauri command and the dev HTTP server
/// (src/devserver.rs) so there is exactly one implementation.
pub(crate) fn run_extract(path: &str) -> Result<Vec<nlp::Annotation>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    Ok(nlp::extract_nouns(&text))
}

/// Extract the L1 annotation layer (noun-ish terms + frequency) for a single
/// Markdown file. Pure read: loads the file, runs the NLP pipeline, returns
/// the ranked annotations. No DB write — the first slice is stateless.
#[tauri::command]
pub async fn extract_annotations(path: String) -> Result<Vec<nlp::Annotation>, String> {
    tokio::task::spawn_blocking(move || run_extract(&path))
        .await
        .map_err(|e| format!("task join: {e}"))?
}
