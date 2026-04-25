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

/// Convert a `.docx` file to Markdown using a native Rust OOXML walker
/// (zip + quick-xml). No external runtime. Returns Markdown plus any
/// embedded media as base64-encoded sidecar assets (same shape as the
/// PPTX converter — frontend writes them under `attachments/<name>`).
#[tauri::command]
pub async fn convert_docx_to_markdown(path: String) -> Result<ConvertResult, String> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    tokio::task::spawn_blocking(move || -> Result<ConvertResult, String> {
        let bytes = std::fs::read(&path).map_err(|e| format!("read {path}: {e}"))?;
        let result = crate::convert::docx::docx_to_markdown(&bytes)
            .map_err(|e| e.to_string())?;
        let assets = result
            .assets
            .into_iter()
            .map(|a| ConvertAsset {
                name: a.name,
                bytes_b64: STANDARD.encode(&a.bytes),
            })
            .collect();
        Ok(ConvertResult {
            markdown: result.markdown,
            assets,
        })
    })
    .await
    .map_err(|e| format!("task join: {e}"))?
}

/// One asset extracted from a converted document — for PPTX today this
/// is an embedded picture from `ppt/media/`. Bytes are base64-encoded so
/// the value can ride a JSON IPC payload; the frontend writes them to
/// disk on Save (under `attachments/<name>`).
#[derive(serde::Serialize)]
pub struct ConvertAsset {
    pub name: String,
    pub bytes_b64: String,
}

/// Convert result for formats that may carry sidecar assets (currently
/// PPTX). The `markdown` already references each asset by its final
/// `attachments/<name>` filename so callers don't need to rewrite text.
#[derive(serde::Serialize)]
pub struct ConvertResult {
    pub markdown: String,
    pub assets: Vec<ConvertAsset>,
}

/// Convert a `.pptx` file to Markdown using a native Rust OOXML walker
/// (zip + quick-xml). No external runtime. Returns Markdown plus any
/// embedded media as base64-encoded sidecar assets.
#[tauri::command]
pub async fn convert_pptx_to_markdown(path: String) -> Result<ConvertResult, String> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    tokio::task::spawn_blocking(move || -> Result<ConvertResult, String> {
        let bytes = std::fs::read(&path).map_err(|e| format!("read {path}: {e}"))?;
        let result = crate::convert::pptx::pptx_to_markdown(&bytes)
            .map_err(|e| e.to_string())?;
        let assets = result
            .assets
            .into_iter()
            .map(|a| ConvertAsset {
                name: a.name,
                bytes_b64: STANDARD.encode(&a.bytes),
            })
            .collect();
        Ok(ConvertResult {
            markdown: result.markdown,
            assets,
        })
    })
    .await
    .map_err(|e| format!("task join: {e}"))?
}
