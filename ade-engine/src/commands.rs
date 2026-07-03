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
