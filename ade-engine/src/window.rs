//! Secondary-window management for the detachable preview (GWEN-489).

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Opens (or focuses) the floating preview window.
///
/// The preview loads the same frontend bundle with a `?preview` flag, so
/// `main.ts` mounts a preview-only root instead of the full shell. If the
/// window already exists (double-click on detach), it's just focused.
#[tauri::command]
pub async fn open_preview_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("preview") {
        let _ = existing.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app,
        "preview",
        WebviewUrl::App("index.html?preview".into()),
    )
    .title("ADE Preview")
    .inner_size(800.0, 600.0)
    .min_inner_size(400.0, 320.0)
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
