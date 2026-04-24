use std::process::Command;

/// Convert an arbitrary document at `path` to Markdown by spawning
/// `uvx --from 'markitdown[all]' markitdown <path>`.
///
/// The `[all]` extras are requested because markitdown's per-format handlers
/// (pdf, docx, pptx, xlsx, image OCR, audio transcribe, etc.) are opt-in via
/// extras. First invocation pays the dependency download cost; subsequent
/// runs reuse the uv tool cache.
#[tauri::command]
pub async fn convert_to_markdown(path: String) -> Result<String, String> {
    let output = Command::new("uvx")
        .args(["--from", "markitdown[all]", "markitdown", &path])
        .output()
        .map_err(|e| {
            format!(
                "Failed to spawn uvx: {e}. Is uv installed? See https://docs.astral.sh/uv/"
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(if stderr.is_empty() {
            format!("markitdown exited with status {}", output.status)
        } else {
            format!("markitdown error:\n{stderr}")
        });
    }
    String::from_utf8(output.stdout)
        .map_err(|e| format!("markitdown output is not valid UTF-8: {e}"))
}

/// Convert an `.html` / `.htm` file to Markdown using the native Rust
/// pipeline (chardetng + dom_smoothie + htmd). No external runtime.
///
/// CPU-bound work is offloaded to tokio's blocking pool so it doesn't
/// starve the Tauri IPC runtime worker.
#[tauri::command]
pub async fn convert_html_to_markdown(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let bytes = std::fs::read(&path).map_err(|e| format!("read {path}: {e}"))?;
        crate::convert::html::html_to_markdown(&bytes).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("task join: {e}"))?
}
