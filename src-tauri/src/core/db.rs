use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::core::blob_store;
use crate::core::marrow_dir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotMeta {
    pub ts: i64,
    pub hash: String,
    pub op: String,
    pub size: i64,
    pub prev_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NoteRec {
    id: Thing,
}

#[derive(Debug, Deserialize)]
struct ExistsRec {
    #[allow(dead_code)]
    id: Thing,
}

#[derive(Clone, Debug)]
pub struct WorkspaceCtx {
    pub workspace_id: String,
    pub root: PathBuf,
}

pub struct DbState {
    pub db: Surreal<Db>,
    pub objects_root: PathBuf,
    pub current: Mutex<Option<WorkspaceCtx>>,
}

impl DbState {
    pub async fn open_global(app: &AppHandle) -> Result<Self, String> {
        marrow_dir::ensure_global_dirs(app)?;
        let db_path = marrow_dir::db_root(app)?;
        let db_path_str = db_path
            .to_str()
            .ok_or_else(|| format!("non-utf8 db path: {}", db_path.display()))?;
        let db: Surreal<Db> = Surreal::new::<SurrealKv>(db_path_str)
            .await
            .map_err(|e| format!("open surrealkv: {e}"))?;
        db.use_ns("marrow")
            .use_db("global")
            .await
            .map_err(|e| format!("use ns/db: {e}"))?;
        let state = Self {
            db,
            objects_root: marrow_dir::objects_root(app)?,
            current: Mutex::new(None),
        };
        state.apply_migrations().await?;
        Ok(state)
    }

    async fn apply_migrations(&self) -> Result<(), String> {
        let ddl = r#"
            DEFINE TABLE IF NOT EXISTS workspace SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS name ON workspace TYPE string;
            DEFINE FIELD IF NOT EXISTS last_path ON workspace TYPE string;
            DEFINE FIELD IF NOT EXISTS created_ts ON workspace TYPE int;
            DEFINE FIELD IF NOT EXISTS last_opened_ts ON workspace TYPE int;

            DEFINE TABLE IF NOT EXISTS note SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS workspace ON note TYPE record<workspace>;
            DEFINE FIELD IF NOT EXISTS path ON note TYPE string;
            DEFINE FIELD IF NOT EXISTS created_ts ON note TYPE int;
            DEFINE INDEX IF NOT EXISTS note_ws_path ON note FIELDS workspace, path UNIQUE;

            DEFINE TABLE IF NOT EXISTS snapshot SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS note ON snapshot TYPE record<note>;
            DEFINE FIELD IF NOT EXISTS ts ON snapshot TYPE int;
            DEFINE FIELD IF NOT EXISTS hash ON snapshot TYPE string;
            DEFINE FIELD IF NOT EXISTS op ON snapshot TYPE string;
            DEFINE FIELD IF NOT EXISTS size ON snapshot TYPE int;
            DEFINE FIELD IF NOT EXISTS prev_path ON snapshot TYPE option<string>;
            DEFINE INDEX IF NOT EXISTS snapshot_note_ts ON snapshot FIELDS note, ts;

            DEFINE TABLE IF NOT EXISTS graph_layout SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS workspace ON graph_layout TYPE record<workspace>;
            DEFINE FIELD IF NOT EXISTS data ON graph_layout FLEXIBLE TYPE object;
            DEFINE FIELD IF NOT EXISTS updated_ts ON graph_layout TYPE int;
            DEFINE INDEX IF NOT EXISTS graph_layout_ws ON graph_layout FIELDS workspace UNIQUE;

            DEFINE TABLE IF NOT EXISTS _meta SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS schema_version ON _meta TYPE int;
        "#;
        self.db
            .query(ddl)
            .await
            .map_err(|e| format!("apply migrations: {e}"))?;
        self.db
            .query("UPSERT _meta:current SET schema_version = 2;")
            .await
            .map_err(|e| format!("seed schema_version: {e}"))?;
        Ok(())
    }

    pub async fn activate_workspace(&self, root: &Path) -> Result<WorkspaceCtx, String> {
        let canonical = root
            .canonicalize()
            .map_err(|e| format!("canonicalize workspace root: {e}"))?;
        let ws_id = marrow_dir::load_or_create_workspace_id(&canonical)?;
        let name = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| canonical.to_string_lossy().to_string());
        let last_path = canonical.to_string_lossy().to_string();
        let now = now_ts();
        let ws_thing = Thing::from(("workspace", ws_id.as_str()));

        // Preserve created_ts: check existence, then UPDATE or CREATE.
        let mut res = self
            .db
            .query("SELECT id FROM $id")
            .bind(("id", ws_thing.clone()))
            .await
            .map_err(|e| format!("select workspace: {e}"))?;
        let existing: Vec<ExistsRec> = res.take(0).map_err(|e| format!("take: {e}"))?;

        if existing.is_empty() {
            self.db
                .query(
                    "CREATE $id SET name = $n, last_path = $lp, created_ts = $now, last_opened_ts = $now",
                )
                .bind(("id", ws_thing.clone()))
                .bind(("n", name))
                .bind(("lp", last_path.clone()))
                .bind(("now", now))
                .await
                .map_err(|e| format!("create workspace: {e}"))?;
        } else {
            self.db
                .query("UPDATE $id SET name = $n, last_path = $lp, last_opened_ts = $now")
                .bind(("id", ws_thing.clone()))
                .bind(("n", name))
                .bind(("lp", last_path))
                .bind(("now", now))
                .await
                .map_err(|e| format!("update workspace: {e}"))?;
        }

        let ctx = WorkspaceCtx {
            workspace_id: ws_id,
            root: canonical,
        };
        *self.current.lock().await = Some(ctx.clone());
        Ok(ctx)
    }

    pub async fn current_ctx(&self) -> Option<WorkspaceCtx> {
        self.current.lock().await.clone()
    }

    pub fn to_rel_path(ctx: &WorkspaceCtx, abs_path: &Path) -> Option<String> {
        abs_path
            .strip_prefix(&ctx.root)
            .ok()
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
    }

    fn ws_thing(ctx: &WorkspaceCtx) -> Thing {
        Thing::from(("workspace", ctx.workspace_id.as_str()))
    }

    async fn ensure_note(&self, ctx: &WorkspaceCtx, rel_path: &str) -> Result<Thing, String> {
        let ws = Self::ws_thing(ctx);
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE workspace = $w AND path = $p")
            .bind(("w", ws.clone()))
            .bind(("p", rel_path.to_string()))
            .await
            .map_err(|e| format!("select note: {e}"))?;
        let existing: Vec<NoteRec> = res.take(0).map_err(|e| format!("take: {e}"))?;
        if let Some(n) = existing.into_iter().next() {
            return Ok(n.id);
        }
        let now = now_ts();
        let mut res = self
            .db
            .query("CREATE note SET workspace = $w, path = $p, created_ts = $t")
            .bind(("w", ws))
            .bind(("p", rel_path.to_string()))
            .bind(("t", now))
            .await
            .map_err(|e| format!("create note: {e}"))?;
        let created: Vec<NoteRec> = res.take(0).map_err(|e| format!("take: {e}"))?;
        created
            .into_iter()
            .next()
            .map(|n| n.id)
            .ok_or_else(|| "create note returned empty".into())
    }

    async fn latest_hash_for(&self, note_id: &Thing) -> Result<Option<String>, String> {
        #[derive(Deserialize)]
        struct Row {
            hash: String,
        }
        let mut res = self
            .db
            .query("SELECT hash FROM snapshot WHERE note = $n ORDER BY ts DESC LIMIT 1")
            .bind(("n", note_id.clone()))
            .await
            .map_err(|e| format!("latest hash query: {e}"))?;
        let rows: Vec<Row> = res.take(0).map_err(|e| format!("take: {e}"))?;
        Ok(rows.into_iter().next().map(|r| r.hash))
    }

    pub async fn snapshot_save(
        &self,
        ctx: &WorkspaceCtx,
        rel_path: &str,
        content: &[u8],
    ) -> Result<(), String> {
        self.snapshot_with_op(ctx, rel_path, content, "save").await
    }

    pub async fn snapshot_restore(
        &self,
        ctx: &WorkspaceCtx,
        rel_path: &str,
        content: &[u8],
    ) -> Result<(), String> {
        self.snapshot_with_op(ctx, rel_path, content, "restore")
            .await
    }

    async fn snapshot_with_op(
        &self,
        ctx: &WorkspaceCtx,
        rel_path: &str,
        content: &[u8],
        op: &str,
    ) -> Result<(), String> {
        let hash = blob_store::write_blob(&self.objects_root, content)
            .map_err(|e| format!("write blob: {e}"))?;
        let note_id = self.ensure_note(ctx, rel_path).await?;
        if op == "save" {
            if let Some(prev) = self.latest_hash_for(&note_id).await? {
                if prev == hash {
                    return Ok(());
                }
            }
        }
        let now = now_ts();
        self.db
            .query("CREATE snapshot SET note = $n, ts = $t, hash = $h, op = $o, size = $s")
            .bind(("n", note_id))
            .bind(("t", now))
            .bind(("h", hash))
            .bind(("o", op.to_string()))
            .bind(("s", content.len() as i64))
            .await
            .map_err(|e| format!("insert snapshot: {e}"))?;
        Ok(())
    }

    pub async fn snapshot_rename(
        &self,
        ctx: &WorkspaceCtx,
        from_rel: &str,
        to_rel: &str,
    ) -> Result<(), String> {
        let ws = Self::ws_thing(ctx);
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE workspace = $w AND path = $p")
            .bind(("w", ws))
            .bind(("p", from_rel.to_string()))
            .await
            .map_err(|e| format!("select note: {e}"))?;
        let existing: Vec<NoteRec> = res.take(0).map_err(|e| format!("take: {e}"))?;
        let Some(note) = existing.into_iter().next() else {
            return Ok(());
        };
        self.db
            .query("UPDATE $id SET path = $p")
            .bind(("id", note.id.clone()))
            .bind(("p", to_rel.to_string()))
            .await
            .map_err(|e| format!("update note path: {e}"))?;
        let now = now_ts();
        self.db
            .query(
                "CREATE snapshot SET note = $n, ts = $t, hash = '', op = 'rename', size = 0, prev_path = $prev",
            )
            .bind(("n", note.id))
            .bind(("t", now))
            .bind(("prev", from_rel.to_string()))
            .await
            .map_err(|e| format!("insert rename snapshot: {e}"))?;
        Ok(())
    }

    pub async fn list_history(
        &self,
        ctx: &WorkspaceCtx,
        rel_path: &str,
    ) -> Result<Vec<SnapshotMeta>, String> {
        let ws = Self::ws_thing(ctx);
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE workspace = $w AND path = $p")
            .bind(("w", ws))
            .bind(("p", rel_path.to_string()))
            .await
            .map_err(|e| format!("select note: {e}"))?;
        let notes: Vec<NoteRec> = res.take(0).map_err(|e| format!("take: {e}"))?;
        let Some(note) = notes.into_iter().next() else {
            return Ok(vec![]);
        };
        let mut res = self
            .db
            .query(
                "SELECT ts, hash, op, size, prev_path FROM snapshot WHERE note = $n ORDER BY ts DESC",
            )
            .bind(("n", note.id))
            .await
            .map_err(|e| format!("list snapshots: {e}"))?;
        let rows: Vec<SnapshotMeta> = res.take(0).map_err(|e| format!("take: {e}"))?;
        Ok(rows)
    }

    pub fn read_blob(&self, hash: &str) -> Result<Vec<u8>, String> {
        blob_store::read_blob(&self.objects_root, hash).map_err(|e| format!("read blob: {e}"))
    }

    pub async fn load_graph_layout(
        &self,
        ctx: &WorkspaceCtx,
    ) -> Result<Option<serde_json::Value>, String> {
        #[derive(Deserialize)]
        struct Row {
            data: serde_json::Value,
        }
        let gl = Thing::from(("graph_layout", ctx.workspace_id.as_str()));
        let mut res = self
            .db
            .query("SELECT data FROM $id")
            .bind(("id", gl))
            .await
            .map_err(|e| format!("select graph_layout: {e}"))?;
        let rows: Vec<Row> = res.take(0).map_err(|e| format!("take: {e}"))?;
        Ok(rows.into_iter().next().map(|r| r.data))
    }

    pub async fn save_graph_layout(
        &self,
        ctx: &WorkspaceCtx,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let ws = Self::ws_thing(ctx);
        let gl = Thing::from(("graph_layout", ctx.workspace_id.as_str()));
        let now = now_ts();
        self.db
            .query("UPSERT $id SET workspace = $w, data = $d, updated_ts = $t")
            .bind(("id", gl))
            .bind(("w", ws))
            .bind(("d", data))
            .bind(("t", now))
            .await
            .map_err(|e| format!("upsert graph_layout: {e}"))?;
        Ok(())
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
