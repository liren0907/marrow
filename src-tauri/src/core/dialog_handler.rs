pub struct DialogHandler;

impl DialogHandler {
    pub fn select_folder() -> Result<String, String> {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            Ok(folder.to_string_lossy().to_string())
        } else {
            Err("No folder selected".into())
        }
    }
}
