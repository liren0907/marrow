use tauri::State;

use crate::core::db::DbState;

#[tauri::command]
pub async fn load_graph_layout(
    db: State<'_, DbState>,
) -> Result<Option<serde_json::Value>, String> {
    let ctx = db.current_ctx().await.ok_or("no workspace open")?;
    db.load_graph_layout(&ctx).await
}

#[tauri::command]
pub async fn save_graph_layout(
    data: serde_json::Value,
    db: State<'_, DbState>,
) -> Result<(), String> {
    let ctx = db.current_ctx().await.ok_or("no workspace open")?;
    db.save_graph_layout(&ctx, data).await
}
