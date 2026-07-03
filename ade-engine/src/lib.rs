pub mod commands;
pub mod memory;
pub mod providers;

use commands::WorkspaceState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Init memory on startup; a failure is non-fatal — the app still runs.
    if let Err(e) = memory::init_memory() {
        eprintln!("Warning: could not init memory: {e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .manage(WorkspaceState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::pick_workspace,
            commands::get_workspace,
            commands::generate,
            commands::record_feedback,
            commands::list_providers,
            commands::save_api_key,
            commands::get_api_key,
            commands::has_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GwenLand ADE");
}
