use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use tauri::State;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::commands::workspace::{search_root_impl, SearchHit};
use crate::core::db::DbState;

const MAX_CONCURRENT: usize = 4;

#[derive(Debug, Serialize)]
pub struct CrossHit {
    pub workspace_id: String,
    pub workspace_name: String,
    pub workspace_root: String,
    pub hit: SearchHit,
}

#[derive(Debug, Deserialize)]
struct Row {
    id: Thing,
    name: String,
    last_path: String,
}

#[tauri::command]
pub async fn search_all_workspaces(
    query: String,
    max_results: Option<usize>,
    db: State<'_, DbState>,
) -> Result<Vec<CrossHit>, String> {
    let trimmed = query.trim().to_string();
    if trimmed.len() < 2 {
        return Ok(vec![]);
    }
    let limit = max_results.unwrap_or(200);

    let mut res = db
        .db
        .query(
            "SELECT id, name, last_path FROM workspace \
             WHERE hidden != true ORDER BY last_opened_ts DESC",
        )
        .await
        .map_err(|e| format!("list workspaces: {e}"))?;
    let rows: Vec<Row> = res.take(0).map_err(|e| format!("take: {e}"))?;

    let workspaces: Vec<(usize, String, String, String)> = rows
        .into_iter()
        .filter(|r| Path::new(&r.last_path).is_dir())
        .enumerate()
        .map(|(i, r)| (i, r.id.id.to_raw(), r.name, r.last_path))
        .collect();

    if workspaces.is_empty() {
        return Ok(vec![]);
    }

    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let query_arc = Arc::new(trimmed);
    let per_ws_limit = limit;

    let mut set: JoinSet<(usize, String, String, String, Result<Vec<SearchHit>, String>)> =
        JoinSet::new();
    for (ord, ws_id, ws_name, root) in workspaces {
        let sem = sem.clone();
        let q = query_arc.clone();
        let root_for_task = root.clone();
        set.spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return (ord, ws_id, ws_name, root, Ok(vec![])),
            };
            let qstr = (*q).clone();
            let r = tokio::task::spawn_blocking(move || {
                search_root_impl(&root_for_task, &qstr, Some(per_ws_limit))
            })
            .await
            .map_err(|e| format!("join: {e}"))
            .and_then(|x| x);
            (ord, ws_id, ws_name, root, r)
        });
    }

    let mut merged: Vec<(usize, CrossHit)> = Vec::new();
    while let Some(joined) = set.join_next().await {
        let (ord, ws_id, ws_name, ws_root, result) = match joined {
            Ok(v) => v,
            Err(_) => continue,
        };
        let hits = match result {
            Ok(h) => h,
            Err(_) => continue,
        };
        for hit in hits {
            merged.push((
                ord,
                CrossHit {
                    workspace_id: ws_id.clone(),
                    workspace_name: ws_name.clone(),
                    workspace_root: ws_root.clone(),
                    hit,
                },
            ));
        }
    }

    merged.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.hit.path.cmp(&b.1.hit.path))
            .then(a.1.hit.line.cmp(&b.1.hit.line))
    });

    let out: Vec<CrossHit> = merged
        .into_iter()
        .take(limit)
        .map(|(_, h)| h)
        .collect();
    Ok(out)
}
