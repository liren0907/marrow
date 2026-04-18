use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

pub fn blob_path(objects_root: &Path, hash: &str) -> PathBuf {
    // Split first 2 chars as subdirectory (git-style) so directories don't
    // grow unbounded. Refuse malformed hashes defensively.
    if hash.len() < 3 {
        return objects_root.join(hash);
    }
    objects_root.join(&hash[..2]).join(&hash[2..])
}

pub fn blob_exists(objects_root: &Path, hash: &str) -> bool {
    blob_path(objects_root, hash).exists()
}

pub fn write_blob(objects_root: &Path, content: &[u8]) -> io::Result<String> {
    let hash = hash_content(content);
    let path = blob_path(objects_root, &hash);
    if path.exists() {
        return Ok(hash);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(hash)
}

pub fn read_blob(objects_root: &Path, hash: &str) -> io::Result<Vec<u8>> {
    fs::read(blob_path(objects_root, hash))
}
