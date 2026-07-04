use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub struct WorkspaceState(pub Mutex<Option<String>>);

#[tauri::command]
pub async fn pick_workspace(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder = app.dialog().file().blocking_pick_folder();

    if let Some(path) = folder {
        let path_str = path.to_string();
        *state.0.lock().unwrap() = Some(path_str.clone());
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_workspace(state: State<'_, WorkspaceState>) -> Result<Option<String>, String> {
    Ok(state.0.lock().unwrap().clone())
}

#[derive(serde::Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub workspace: Option<String>,
}

#[tauri::command]
pub async fn generate(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
    request: GenerateRequest,
) -> Result<(), String> {
    let workspace = request
        .workspace
        .or_else(|| state.0.lock().unwrap().clone())
        .unwrap_or_else(|| "~".to_string());

    // Build system prompt, then append the memory block (GWEN-484).
    let mut system_prompt = format!(
        "You are ADE (Agentic Development Environment) by GwenLand.\n\
         You are an agentic coding assistant. The user will describe what they want to create or build.\n\
         Current workspace: {workspace}\n\
         Be concise, practical, and agentic. Decide what needs to be done and do it.\n\
         Respond in markdown.",
    );
    let memory = crate::memory::context_block();
    if !memory.is_empty() {
        system_prompt.push_str("\n\n");
        system_prompt.push_str(&memory);
    }

    // TODO: integrate dengan GwenLand provider registry. Until the provider
    // consumes `system_prompt`, keep the assembled prompt observable so the
    // memory injection (GWEN-484) is verifiable end-to-end.
    eprintln!(
        "[generate] system prompt assembled ({} bytes)",
        system_prompt.len()
    );

    // Untuk sekarang, emit placeholder streaming
    let tokens = vec![
        "Got it! ",
        "Let me ",
        "work on ",
        "that for ",
        "you...\n\n",
        "```rust\n",
        "// Your code here\n",
        "```",
    ];

    for token in tokens {
        app.emit("ade://token", token).map_err(|e| e.to_string())?;
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    }

    app.emit("ade://done", "").map_err(|e| e.to_string())?;

    // Reflection is now driven by explicit user feedback via `record_feedback`
    // (GWEN-485), rather than guessed at the end of a stubbed run.
    Ok(())
}

/// User's verdict on an ADE response, sent from the feedback UI (GWEN-485).
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackAction {
    /// The user accepted the output as-is.
    Accept,
    /// The user rejected the output.
    Reject,
    /// The user supplied a corrected version (carried in `tweak`).
    Tweak,
}

#[derive(Debug, serde::Deserialize)]
pub struct FeedbackRequest {
    pub action: FeedbackAction,
    /// The original prompt that produced the output.
    pub prompt: String,
    /// The ADE output the user is judging.
    pub output: String,
    /// The corrected text, present when `action` is `Tweak`.
    #[serde(default)]
    pub tweak: Option<String>,
}

/// Records user feedback on an ADE response and reflects it into memory.
///
/// This is the real signal source for the reflection seam (GWEN-483): it builds
/// a [`TaskOutcome`] from the user's verdict and calls `memory::reflect`. The
/// preference/failure extraction is heuristic for now (GWEN-486, GWEN-487) with
/// a seam for model judgment later. Never fails the caller over a memory write.
#[tauri::command]
pub async fn record_feedback(request: FeedbackRequest) -> Result<(), String> {
    use crate::memory;

    let outcome = match request.action {
        // Accept is signal too — but nothing to record yet.
        FeedbackAction::Accept => memory::TaskOutcome::default(),
        FeedbackAction::Reject => memory::TaskOutcome {
            errored: true,
            failure_summary: memory::judge_failure(&request.prompt, &request.output),
            user_correction: None,
        },
        FeedbackAction::Tweak => memory::TaskOutcome {
            errored: false,
            failure_summary: None,
            user_correction: memory::extract_preference(&request.output, request.tweak.as_deref()),
        },
    };

    if let Err(e) = memory::reflect(&outcome) {
        // Non-fatal: surface to logs, but don't fail the UI action.
        eprintln!("Warning: reflection failed: {e}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Memory panel: first-time detection + file read/write (GWEN-490, GWEN-491)
// ---------------------------------------------------------------------------

/// Best-effort local username for the composer greeting.
#[tauri::command]
pub fn get_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default()
}

/// True once ADE's memory has been initialized (GWEN-490).
#[tauri::command]
pub fn has_memory() -> bool {
    crate::memory::has_memory()
}

/// Reads a memory file (`failures.md` / `preferences.md`) for the viewer.
/// Returns an empty string if the file doesn't exist yet.
#[tauri::command]
pub fn read_memory_file(filename: String) -> Result<String, String> {
    crate::memory::read_memory_file(&filename).map_err(|e| e.to_string())
}

/// Writes edited memory content back to disk (GWEN-491).
#[tauri::command]
pub fn write_memory_file(filename: String, content: String) -> Result<(), String> {
    crate::memory::write_memory_file(&filename, &content).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Memory export / import (GWEN-492)
// ---------------------------------------------------------------------------

/// Zips every `*.md` in the memory dir and writes it to a user-chosen path.
/// Returns `Ok(false)` if the user cancelled the save dialog.
#[tauri::command]
pub async fn export_memory(app: AppHandle) -> Result<bool, String> {
    use std::io::Write;
    use tauri_plugin_dialog::DialogExt;

    let dir = crate::memory::memory_dir().map_err(|e| e.to_string())?;
    let files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();

    let mut zip_buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
        let options = zip::write::SimpleFileOptions::default();
        for path in &files {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("bad filename")?
                .to_string();
            let content = std::fs::read(path).map_err(|e| e.to_string())?;
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            zip.write_all(&content).map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
    }

    let save_path = app
        .dialog()
        .file()
        .add_filter("ZIP", &["zip"])
        .set_file_name("ade-memory-backup.zip")
        .blocking_save_file();

    let Some(save_path) = save_path else {
        return Ok(false); // user cancelled
    };
    let path = save_path.into_path().map_err(|e| e.to_string())?;
    std::fs::write(path, zip_buf).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Picks a `.zip` and extracts its `*.md` entries into the memory dir,
/// overwriting existing files. Returns `Ok(false)` if the user cancelled.
#[tauri::command]
pub async fn import_memory(app: AppHandle) -> Result<bool, String> {
    use std::io::Read;
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .add_filter("ZIP", &["zip"])
        .blocking_pick_file();

    let Some(picked) = picked else {
        return Ok(false); // user cancelled
    };
    let zip_path = picked.into_path().map_err(|e| e.to_string())?;

    let dir = crate::memory::memory_dir().map_err(|e| e.to_string())?;
    let data = std::fs::read(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(data)).map_err(|e| e.to_string())?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        // Only accept flat `*.md` names — reject paths that would escape the dir.
        if !name.ends_with(".md") || name.contains(['/', '\\']) || name.contains("..") {
            continue;
        }
        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&name), content).map_err(|e| e.to_string())?;
    }

    Ok(true)
}

// ---------------------------------------------------------------------------
// Provider registry + API-key storage (GWEN-464..469)
// ---------------------------------------------------------------------------

/// Keychain service name under which ADE stores provider API keys.
const KEYCHAIN_SERVICE: &str = "dev.gwenland.ade";

/// Returns the full provider registry so the Settings UI can render key inputs
/// and the model selector data-driven (GWEN-469).
#[tauri::command]
pub fn list_providers() -> Vec<serde_json::Value> {
    crate::providers::registry()
        .iter()
        .map(|p| {
            let models: Vec<_> = p
                .models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "displayName": m.display_name,
                        "contextWindow": m.context_window,
                        "inputPrice": m.input_price,
                        "outputPrice": m.output_price,
                        "display": m.display(),
                    })
                })
                .collect();
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "apiKeyEnv": p.api_key_env,
                "baseUrl": p.base_url,
                "kind": p.kind,
                "models": models,
            })
        })
        .collect()
}

/// Saves a provider's API key to the OS keychain. An empty key deletes the entry.
#[tauri::command]
pub fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &provider).map_err(|e| e.to_string())?;
    if key.trim().is_empty() {
        // Treat "clear" as delete; ignore "not found".
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    } else {
        entry.set_password(&key).map_err(|e| e.to_string())
    }
}

/// Returns the stored API key for a provider, falling back to its env var.
/// The key is returned to the frontend so it can call the provider directly.
#[tauri::command]
pub fn get_api_key(provider: String) -> Result<Option<String>, String> {
    if let Some(p) = crate::providers::find(&provider) {
        if let Ok(v) = std::env::var(p.api_key_env) {
            if !v.trim().is_empty() {
                return Ok(Some(v));
            }
        }
    }
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &provider).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Whether a provider has a usable key (env var or keychain) — lets Settings show
/// a "configured" state without exposing the secret.
#[tauri::command]
pub fn has_api_key(provider: String) -> bool {
    get_api_key(provider).ok().flatten().is_some()
}
