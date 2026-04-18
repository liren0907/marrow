use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub fn app_data_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {e}"))
}

pub fn db_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_root(app)?.join("db"))
}

pub fn objects_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_root(app)?.join("objects"))
}

pub fn ensure_global_dirs(app: &AppHandle) -> Result<(), String> {
    fs::create_dir_all(db_root(app)?).map_err(|e| format!("mkdir db: {e}"))?;
    fs::create_dir_all(objects_root(app)?).map_err(|e| format!("mkdir objects: {e}"))?;
    Ok(())
}

pub fn marker_file(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".marrow-id")
}

pub fn load_or_create_workspace_id(workspace_root: &Path) -> Result<String, String> {
    let path = marker_file(workspace_root);
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| format!("read .marrow-id: {e}"))?;
        let trimmed = raw.trim();
        match Uuid::parse_str(trimmed) {
            Ok(u) => return Ok(u.to_string()),
            Err(e) => {
                eprintln!(
                    "[marrow] .marrow-id is malformed ({e}); regenerating at {}",
                    path.display()
                );
            }
        }
    }
    let new_id = Uuid::new_v4().to_string();
    let tmp = path.with_extension("id.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| format!("create .marrow-id.tmp: {e}"))?;
        writeln!(f, "{new_id}").map_err(|e| format!("write .marrow-id: {e}"))?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &path).map_err(|e| format!("rename .marrow-id: {e}"))?;
    Ok(new_id)
}
