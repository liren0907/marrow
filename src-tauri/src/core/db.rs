use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::sql::Thing;
use surrealdb::Surreal;
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

pub struct DbState {
    pub db: Surreal<Db>,
    pub workspace_root: PathBuf,
}

pub struct DbHandle(pub Mutex<Option<Arc<DbState>>>);

impl DbHandle {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub async fn replace(&self, new: Arc<DbState>) {
        let mut guard = self.0.lock().await;
        *guard = Some(new);
    }

    pub async fn current(&self) -> Option<Arc<DbState>> {
        self.0.lock().await.clone()
    }
}

impl DbState {
    pub async fn open(workspace_root: &Path) -> Result<Self, String> {
        marrow_dir::ensure_exists(workspace_root)
            .map_err(|e| format!("ensure .marrow dirs: {e}"))?;
        let db_path = marrow_dir::db_root(workspace_root);
        let db_path_str = db_path
            .to_str()
            .ok_or_else(|| format!("non-utf8 db path: {}", db_path.display()))?;
        let db: Surreal<Db> = Surreal::new::<SurrealKv>(db_path_str)
            .await
            .map_err(|e| format!("open surrealkv: {e}"))?;
        db.use_ns("marrow")
            .use_db("workspace")
            .await
            .map_err(|e| format!("use ns/db: {e}"))?;
        let state = Self {
            db,
            workspace_root: workspace_root.to_path_buf(),
        };
        state.apply_migrations().await?;
        Ok(state)
    }

    async fn apply_migrations(&self) -> Result<(), String> {
        let ddl = r#"
            DEFINE TABLE IF NOT EXISTS note SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS path ON note TYPE string;
            DEFINE FIELD IF NOT EXISTS created_ts ON note TYPE int;
            DEFINE INDEX IF NOT EXISTS note_path ON note FIELDS path UNIQUE;

            DEFINE TABLE IF NOT EXISTS snapshot SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS note ON snapshot TYPE record<note>;
            DEFINE FIELD IF NOT EXISTS ts ON snapshot TYPE int;
            DEFINE FIELD IF NOT EXISTS hash ON snapshot TYPE string;
            DEFINE FIELD IF NOT EXISTS op ON snapshot TYPE string;
            DEFINE FIELD IF NOT EXISTS size ON snapshot TYPE int;
            DEFINE FIELD IF NOT EXISTS prev_path ON snapshot TYPE option<string>;
            DEFINE INDEX IF NOT EXISTS snapshot_note_ts ON snapshot FIELDS note, ts;

            DEFINE TABLE IF NOT EXISTS _meta SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS schema_version ON _meta TYPE int;
        "#;
        self.db
            .query(ddl)
            .await
            .map_err(|e| format!("apply migrations: {e}"))?;
        self.db
            .query("UPSERT _meta:current SET schema_version = 1;")
            .await
            .map_err(|e| format!("seed schema_version: {e}"))?;
        Ok(())
    }

    pub fn objects_root(&self) -> PathBuf {
        marrow_dir::objects_root(&self.workspace_root)
    }

    pub fn to_rel_path(&self, abs_path: &Path) -> Option<String> {
        abs_path
            .strip_prefix(&self.workspace_root)
            .ok()
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
    }

    async fn ensure_note(&self, rel_path: &str) -> Result<Thing, String> {
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE path = $p")
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
            .query("CREATE note SET path = $p, created_ts = $t")
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

    pub async fn snapshot_save(&self, rel_path: &str, content: &[u8]) -> Result<(), String> {
        self.snapshot_with_op(rel_path, content, "save").await
    }

    pub async fn snapshot_restore(&self, rel_path: &str, content: &[u8]) -> Result<(), String> {
        self.snapshot_with_op(rel_path, content, "restore").await
    }

    async fn snapshot_with_op(
        &self,
        rel_path: &str,
        content: &[u8],
        op: &str,
    ) -> Result<(), String> {
        let objects = self.objects_root();
        let hash = blob_store::write_blob(&objects, content)
            .map_err(|e| format!("write blob: {e}"))?;
        let note_id = self.ensure_note(rel_path).await?;
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

    pub async fn snapshot_rename(&self, from_rel: &str, to_rel: &str) -> Result<(), String> {
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE path = $p")
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

    pub async fn list_history(&self, rel_path: &str) -> Result<Vec<SnapshotMeta>, String> {
        let mut res = self
            .db
            .query("SELECT * FROM note WHERE path = $p")
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
        blob_store::read_blob(&self.objects_root(), hash)
            .map_err(|e| format!("read blob: {e}"))
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
