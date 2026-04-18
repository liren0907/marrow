use std::io;
use std::path::{Path, PathBuf};

pub fn marrow_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".marrow")
}

pub fn history_root(workspace_root: &Path) -> PathBuf {
    marrow_root(workspace_root).join("history")
}

pub fn objects_root(workspace_root: &Path) -> PathBuf {
    history_root(workspace_root).join("objects")
}

pub fn db_root(workspace_root: &Path) -> PathBuf {
    history_root(workspace_root).join("db")
}

pub fn ensure_exists(workspace_root: &Path) -> io::Result<()> {
    std::fs::create_dir_all(objects_root(workspace_root))?;
    std::fs::create_dir_all(db_root(workspace_root))?;
    Ok(())
}

pub fn is_under_marrow(workspace_root: &Path, path: &Path) -> bool {
    path.starts_with(marrow_root(workspace_root))
}
