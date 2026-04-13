use crate::core::dialog_handler::DialogHandler;

#[tauri::command]
pub fn open_directory_dialog() -> Result<String, String> {
    DialogHandler::select_folder()
}
