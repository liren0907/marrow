use std::path::Path;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tauri::State;

use crate::core::db::DbState;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub last_path: String,
    pub last_opened_ts: i64,
    pub created_ts: i64,
}

#[derive(Debug, Deserialize)]
struct Row {
    id: Thing,
    name: String,
    last_path: String,
    last_opened_ts: i64,
    created_ts: i64,
}

#[tauri::command]
pub async fn list_recent_workspaces(
    limit: u32,
    db: State<'_, DbState>,
) -> Result<Vec<WorkspaceSummary>, String> {
    let mut res = db
        .db
        .query(
            "SELECT id, name, last_path, last_opened_ts, created_ts FROM workspace \
             WHERE hidden != true ORDER BY last_opened_ts DESC LIMIT $l",
        )
        .bind(("l", limit as i64))
        .await
        .map_err(|e| format!("list recent workspaces: {e}"))?;
    let rows: Vec<Row> = res.take(0).map_err(|e| format!("take: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| WorkspaceSummary {
            id: r.id.id.to_raw(),
            name: r.name,
            last_path: r.last_path,
            last_opened_ts: r.last_opened_ts,
            created_ts: r.created_ts,
        })
        .collect())
}

#[tauri::command]
pub async fn forget_workspace(id: String, db: State<'_, DbState>) -> Result<(), String> {
    let ws = Thing::from(("workspace", id.as_str()));
    db.db
        .query("UPDATE $id SET hidden = true")
        .bind(("id", ws))
        .await
        .map_err(|e| format!("forget workspace: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}
