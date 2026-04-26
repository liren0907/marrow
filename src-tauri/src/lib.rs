pub mod commands;
pub mod core;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Shared app config — both the watcher (live debounce/TTL
            // reads) and the `set_app_config` command (writes) hold a
            // clone of the same Arc<RwLock>.
            let shared_cfg = core::app_config::new_shared();
            app.manage(core::app_config::AppConfigState(shared_cfg.clone()));
            app.manage(core::fs_watch::WatcherState::new(shared_cfg));
            let handle = app.handle().clone();
            let db = tauri::async_runtime::block_on(core::db::DbState::open_global(&handle))
                .expect("open global db");
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::open_workspace,
            commands::workspace::list_directory,
            commands::workspace::list_workspace_files,
            commands::workspace::search_workspace,
            commands::search_all::search_all_workspaces,
            commands::workspace::read_text_file,
            commands::workspace::read_binary_file,
            commands::workspace::write_text_file,
            commands::workspace::write_binary_file,
            commands::workspace::create_file,
            commands::workspace::create_directory,
            commands::workspace::delete_path,
            commands::workspace::rename_path,
            commands::dialog::open_directory_dialog,
            commands::history::list_file_history,
            commands::history::read_snapshot,
            commands::history::restore_snapshot,
            commands::graph::load_graph_layout,
            commands::graph::save_graph_layout,
            commands::recent::list_recent_workspaces,
            commands::recent::forget_workspace,
            commands::recent::path_exists,
            commands::git::get_git_branch,
            commands::git::get_watcher_status,
            commands::convert::convert_to_markdown,
            commands::convert::convert_html_to_markdown,
            commands::convert::convert_docx_to_markdown,
            commands::convert::convert_pptx_to_markdown,
            core::app_config::set_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
