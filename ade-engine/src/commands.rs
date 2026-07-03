use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub struct WorkspaceState(pub Mutex<Option<String>>);

#[tauri::command]
pub async fn pick_workspace(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();

    if let Some(path) = folder {
        let path_str = path.to_string();
        *state.0.lock().unwrap() = Some(path_str.clone());
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn get_workspace(
    state: State<'_, WorkspaceState>,
) -> Result<Option<String>, String> {
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
    let workspace = request.workspace
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
        "Got it! ", "Let me ", "work on ", "that for ", "you...\n\n",
        "```rust\n", "// Your code here\n", "```",
    ];

    for token in tokens {
        app.emit("ade://token", token).map_err(|e| e.to_string())?;
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    }

    app.emit("ade://done", "").map_err(|e| e.to_string())?;

    // Post-task reflection (GWEN-483). Non-fatal: never fail the task over a
    // memory write. The stub run has no real error/correction signal yet, so
    // this records nothing until the provider surfaces one.
    let outcome = crate::memory::TaskOutcome::default();
    if let Err(e) = crate::memory::reflect(&outcome) {
        eprintln!("Warning: reflection failed: {e}");
    }

    Ok(())
}
