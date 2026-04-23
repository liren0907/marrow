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
