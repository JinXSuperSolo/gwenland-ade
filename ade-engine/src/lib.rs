pub mod commands;
pub mod memory;
pub mod providers;
pub mod tools;
pub mod window;

use commands::WorkspaceState;
use std::sync::Mutex;
use tauri::{Emitter, Manager, WindowEvent};

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
        .plugin(tauri_plugin_opener::init())
        .manage(WorkspaceState(Mutex::new(None)))
        .manage(tools::AskState::default())
        .on_window_event(|window, event| {
            // When the detached preview window closes, tell the main window so
            // it can restore the split layout (GWEN-489).
            if window.label() == "preview" {
                if let WindowEvent::Destroyed = event {
                    if let Some(main) = window.app_handle().get_webview_window("main") {
                        let _ = main.emit("ade://preview-closed", ());
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::pick_workspace,
            commands::get_workspace,
            commands::generate,
            commands::record_feedback,
            commands::list_providers,
            commands::save_api_key,
            commands::get_api_key,
            commands::has_api_key,
            commands::get_username,
            commands::has_memory,
            commands::read_memory_file,
            commands::write_memory_file,
            commands::export_memory,
            commands::import_memory,
            window::open_preview_window,
            // GL_ agent tool suite (M5).
            tools::gl_read_file,
            tools::gl_write_file,
            tools::gl_edit_file,
            tools::gl_delete_file,
            tools::gl_list_dir,
            tools::gl_grep,
            tools::gl_glob,
            tools::gl_git_diff,
            tools::gl_bash,
            tools::gl_diagnostics,
            tools::gl_ask_user,
            tools::gl_answer_user,
            tools::gl_open_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GwenLand ADE");
}
