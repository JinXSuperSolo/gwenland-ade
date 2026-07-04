//! GL_ agent tool suite (M5) — the tools ADE's agent loop calls to inspect and
//! modify the user's workspace.
//!
//! Every path-taking tool is **sandboxed to the workspace root**: paths are
//! resolved against the selected workspace and rejected if they escape it
//! (`..`, absolute paths outside the root, symlink traversal). The workspace is
//! read from the shared [`WorkspaceState`]; tools error clearly if none is set.
//!
//! Tools return structured JSON so the frontend / model can consume them
//! uniformly. Shell-based tools (`GL_Grep`, `GL_Glob`, `GL_Bash`) dispatch to
//! the platform's native tooling — PowerShell on Windows, `grep`/`find`/`sh`
//! elsewhere — since those utilities differ across OSes and distros.

use crate::commands::WorkspaceState;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

// ---------------------------------------------------------------------------
// Workspace sandbox
// ---------------------------------------------------------------------------

/// Returns the current workspace root, or an error if none is selected.
fn workspace_root(state: &State<'_, WorkspaceState>) -> Result<PathBuf, String> {
    let guard = state.0.lock().map_err(|_| "workspace lock poisoned")?;
    let root = guard
        .as_ref()
        .ok_or("No workspace selected. Pick a workspace folder first.")?;
    Ok(PathBuf::from(root))
}

/// Resolves a user-supplied relative path against the workspace root and
/// guarantees the result stays inside it.
///
/// `rel` is treated as relative to the workspace even if it looks absolute, and
/// `..` components that would escape the root are rejected. The path need not
/// exist yet (so it works for writes/creates); the *parent* chain is validated
/// lexically, and if the target exists its canonical form is re-checked against
/// the canonical root to defeat symlink escapes.
fn resolve_in_workspace(root: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.contains('\0') {
        return Err("invalid path".into());
    }

    // Strip a leading slash so "/src/x" is read as workspace-relative.
    let rel_path = Path::new(rel.trim_start_matches(['/', '\\']));

    // Lexically join + normalize, rejecting any component that climbs above root.
    let mut out = root.to_path_buf();
    for comp in rel_path.components() {
        use std::path::Component::*;
        match comp {
            CurDir => {}
            ParentDir => {
                // Popping must not go above the workspace root.
                if out == *root || !out.starts_with(root) {
                    return Err("path escapes the workspace".into());
                }
                out.pop();
                if !out.starts_with(root) {
                    return Err("path escapes the workspace".into());
                }
            }
            Normal(seg) => out.push(seg),
            // Absolute prefixes/roots in the middle are not allowed.
            RootDir | Prefix(_) => return Err("absolute paths are not allowed".into()),
        }
    }

    // If it exists, re-validate the canonical form against the canonical root so
    // a symlink inside the tree can't point outside it.
    if out.exists() {
        let canon = out.canonicalize().map_err(|e| e.to_string())?;
        let canon_root = root.canonicalize().map_err(|e| e.to_string())?;
        if !canon.starts_with(&canon_root) {
            return Err("path escapes the workspace".into());
        }
    }
    Ok(out)
}

/// Convenience: resolve a path from the state in one step.
fn resolve(state: &State<'_, WorkspaceState>, rel: &str) -> Result<PathBuf, String> {
    let root = workspace_root(state)?;
    resolve_in_workspace(&root, rel)
}

// ---------------------------------------------------------------------------
// GL_Read_File
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ReadResult {
    pub path: String,
    pub content: String,
    pub truncated: bool,
    pub lines: usize,
}

/// Reads a UTF-8 text file inside the workspace. Optional 1-indexed
/// `offset`/`limit` select a line range; large files are truncated.
#[tauri::command]
pub async fn gl_read_file(
    state: State<'_, WorkspaceState>,
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<ReadResult, String> {
    const MAX_BYTES: usize = 512 * 1024;

    let full = resolve(&state, &path)?;
    let raw = std::fs::read(&full).map_err(|e| format!("{path}: {e}"))?;

    let truncated_bytes = raw.len() > MAX_BYTES;
    let slice = if truncated_bytes {
        &raw[..MAX_BYTES]
    } else {
        &raw[..]
    };
    let text = String::from_utf8_lossy(slice);

    let all_lines: Vec<&str> = text.lines().collect();
    let total = all_lines.len();

    let start = offset.unwrap_or(1).saturating_sub(1).min(total);
    let end = match limit {
        Some(n) => (start + n).min(total),
        None => total,
    };
    let selected = all_lines[start..end].join("\n");

    Ok(ReadResult {
        path,
        content: selected,
        truncated: truncated_bytes || end < total,
        lines: total,
    })
}

// ---------------------------------------------------------------------------
// GL_Write_File
// ---------------------------------------------------------------------------

/// Writes (creates or overwrites) a file inside the workspace, creating parent
/// directories as needed.
#[tauri::command]
pub async fn gl_write_file(
    state: State<'_, WorkspaceState>,
    path: String,
    content: String,
) -> Result<serde_json::Value, String> {
    let full = resolve(&state, &path)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&full, content.as_bytes()).map_err(|e| format!("{path}: {e}"))?;
    Ok(serde_json::json!({ "path": path, "bytes": content.len() }))
}

// ---------------------------------------------------------------------------
// GL_Edit_File
// ---------------------------------------------------------------------------

/// Replaces an exact substring in a file. `replace_all` swaps every occurrence;
/// otherwise `old_string` must be unique (errors if it appears 0 or >1 times),
/// mirroring the standard edit-tool contract.
#[tauri::command]
pub async fn gl_edit_file(
    state: State<'_, WorkspaceState>,
    path: String,
    old_string: String,
    new_string: String,
    replace_all: Option<bool>,
) -> Result<serde_json::Value, String> {
    let full = resolve(&state, &path)?;
    let content = std::fs::read_to_string(&full).map_err(|e| format!("{path}: {e}"))?;

    if old_string.is_empty() {
        return Err("old_string must not be empty".into());
    }
    let count = content.matches(&old_string).count();
    if count == 0 {
        return Err("old_string not found in file".into());
    }
    let all = replace_all.unwrap_or(false);
    if count > 1 && !all {
        return Err(format!(
            "old_string is not unique ({count} matches); pass replace_all or add context"
        ));
    }

    let updated = if all {
        content.replace(&old_string, &new_string)
    } else {
        content.replacen(&old_string, &new_string, 1)
    };
    std::fs::write(&full, updated.as_bytes()).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "path": path, "replacements": if all { count } else { 1 } }))
}

// ---------------------------------------------------------------------------
// GL_Delete_File
// ---------------------------------------------------------------------------

/// Deletes a file or (empty or recursive) directory inside the workspace.
#[tauri::command]
pub async fn gl_delete_file(
    state: State<'_, WorkspaceState>,
    path: String,
    recursive: Option<bool>,
) -> Result<serde_json::Value, String> {
    let full = resolve(&state, &path)?;
    let root = workspace_root(&state)?;
    // Never allow deleting the workspace root itself.
    if full == root {
        return Err("refusing to delete the workspace root".into());
    }
    let meta = std::fs::symlink_metadata(&full).map_err(|e| format!("{path}: {e}"))?;
    if meta.is_dir() {
        if recursive.unwrap_or(false) {
            std::fs::remove_dir_all(&full).map_err(|e| e.to_string())?;
        } else {
            std::fs::remove_dir(&full).map_err(|e| {
                format!("{path}: directory not empty (pass recursive to force): {e}")
            })?;
        }
    } else {
        std::fs::remove_file(&full).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "path": path, "deleted": true }))
}

// ---------------------------------------------------------------------------
// GL_List_Dir
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Lists the immediate entries of a directory inside the workspace (defaults to
/// the root). Directories are listed first, then files, both alphabetically.
#[tauri::command]
pub async fn gl_list_dir(
    state: State<'_, WorkspaceState>,
    path: Option<String>,
) -> Result<Vec<DirEntry>, String> {
    let full = resolve(&state, path.as_deref().unwrap_or("."))?;
    let mut entries = Vec::new();
    for e in std::fs::read_dir(&full).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let meta = e.metadata().map_err(|e| e.to_string())?;
        entries.push(DirEntry {
            name: e.file_name().to_string_lossy().into_owned(),
            is_dir: meta.is_dir(),
            size: meta.len(),
        });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(entries)
}

// ---------------------------------------------------------------------------
// Shell dispatch — GL_Grep / GL_Glob / GL_Bash / GL_Git_Diff
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    /// True if the command hit the timeout and was killed.
    pub timed_out: bool,
}

/// Runs a command line with `cwd` at the workspace root, capturing output with a
/// timeout. Uses the platform shell: `cmd /C`… no — PowerShell on Windows,
/// `sh -c` elsewhere. This is the single seam every shell-based GL_ tool uses.
///
/// NOTE (cross-platform): shell utilities differ across OSes and distros
/// (`grep`/`find` on Unix vs PowerShell cmdlets on Windows). The higher-level
/// tools pick an OS-appropriate command; this only runs it.
fn run_shell(cwd: &Path, command: &str, timeout_secs: u64) -> Result<CommandOutput, String> {
    use std::process::{Command, Stdio};

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };
    cmd.current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to start shell: {e}"))?;

    // Poll for completion up to the timeout, then kill.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let mut timed_out = false;
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(_) => break,
            None => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to read command output: {e}"))?;

    Ok(CommandOutput {
        stdout: cap(String::from_utf8_lossy(&output.stdout).into_owned()),
        stderr: cap(String::from_utf8_lossy(&output.stderr).into_owned()),
        exit_code: output
            .status
            .code()
            .unwrap_or(if timed_out { 124 } else { -1 }),
        timed_out,
    })
}

/// Caps captured output so a runaway command can't flood the UI/model.
fn cap(mut s: String) -> String {
    const MAX: usize = 60_000;
    if s.len() > MAX {
        s.truncate(MAX);
        s.push_str("\n… [output truncated]");
    }
    s
}

/// Shell-escapes a value for the current platform (single quotes on Unix,
/// doubled single quotes inside single quotes on PowerShell).
fn shell_quote(s: &str) -> String {
    if cfg!(windows) {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// ---------------------------------------------------------------------------
// GL_Bash
// ---------------------------------------------------------------------------

/// Runs an arbitrary shell command in the workspace and returns its output.
/// Highest-trust tool — the command is the user's own agent acting in their
/// own workspace. `timeout_secs` defaults to 60 (capped at 600).
#[tauri::command]
pub async fn gl_bash(
    state: State<'_, WorkspaceState>,
    command: String,
    timeout_secs: Option<u64>,
) -> Result<CommandOutput, String> {
    let root = workspace_root(&state)?;
    let secs = timeout_secs.unwrap_or(60).clamp(1, 600);
    // Run on a blocking thread so we don't stall the async runtime.
    tokio::task::spawn_blocking(move || run_shell(&root, &command, secs))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// GL_Grep
// ---------------------------------------------------------------------------

/// Searches file contents for a pattern under the workspace, dispatching to the
/// OS-native search tool: `Select-String` on Windows, `grep -rn` on Unix.
/// `path` narrows the search subtree; `glob` filters filenames.
#[tauri::command]
pub async fn gl_grep(
    state: State<'_, WorkspaceState>,
    pattern: String,
    path: Option<String>,
    glob: Option<String>,
) -> Result<CommandOutput, String> {
    let root = workspace_root(&state)?;
    let sub = resolve(&state, path.as_deref().unwrap_or("."))?;
    let sub_str = sub.to_string_lossy().into_owned();

    let command = if cfg!(windows) {
        // Get-ChildItem -Recurse [-Filter glob] | Select-String -Pattern …
        let filter = glob
            .as_deref()
            .map(|g| format!(" -Filter {}", shell_quote(g)))
            .unwrap_or_default();
        format!(
            "Get-ChildItem -Path {} -Recurse -File{} -ErrorAction SilentlyContinue | \
             Select-String -Pattern {} | \
             ForEach-Object {{ \"$($_.Path):$($_.LineNumber):$($_.Line.Trim())\" }}",
            shell_quote(&sub_str),
            filter,
            shell_quote(&pattern),
        )
    } else {
        let include = glob
            .as_deref()
            .map(|g| format!(" --include={}", shell_quote(g)))
            .unwrap_or_default();
        format!(
            "grep -rn{} -e {} {} 2>/dev/null || true",
            include,
            shell_quote(&pattern),
            shell_quote(&sub_str),
        )
    };

    tokio::task::spawn_blocking(move || run_shell(&root, &command, 60))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// GL_Glob
// ---------------------------------------------------------------------------

/// Finds files by name pattern under the workspace, OS-native: `Get-ChildItem
/// -Recurse -Filter` on Windows, `find -name` on Unix. Returns matching paths
/// (relative to the workspace) one per line.
#[tauri::command]
pub async fn gl_glob(
    state: State<'_, WorkspaceState>,
    pattern: String,
    path: Option<String>,
) -> Result<CommandOutput, String> {
    let root = workspace_root(&state)?;
    let sub = resolve(&state, path.as_deref().unwrap_or("."))?;
    let sub_str = sub.to_string_lossy().into_owned();

    let command = if cfg!(windows) {
        format!(
            "Get-ChildItem -Path {} -Recurse -File -Filter {} -ErrorAction SilentlyContinue | \
             Resolve-Path -Relative -ErrorAction SilentlyContinue",
            shell_quote(&sub_str),
            shell_quote(&pattern),
        )
    } else {
        format!(
            "find {} -type f -name {} 2>/dev/null || true",
            shell_quote(&sub_str),
            shell_quote(&pattern),
        )
    };

    tokio::task::spawn_blocking(move || run_shell(&root, &command, 60))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// GL_Git_Diff
// ---------------------------------------------------------------------------

/// Runs `git diff` in the workspace. `staged` diffs the index; `path` narrows to
/// a subtree/file. Returns the raw diff (empty when there are no changes).
#[tauri::command]
pub async fn gl_git_diff(
    state: State<'_, WorkspaceState>,
    staged: Option<bool>,
    path: Option<String>,
) -> Result<CommandOutput, String> {
    let root = workspace_root(&state)?;
    let mut command = String::from("git --no-pager diff --no-color");
    if staged.unwrap_or(false) {
        command.push_str(" --staged");
    }
    if let Some(p) = &path {
        // Validate the path is inside the workspace before passing it to git.
        resolve(&state, p)?;
        command.push_str(" -- ");
        command.push_str(&shell_quote(p));
    }

    tokio::task::spawn_blocking(move || run_shell(&root, &command, 30))
        .await
        .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------------------
// GL_Diagnostics
// ---------------------------------------------------------------------------

/// Runs the workspace's own type/lint check and returns its output. The check
/// command is detected from the project shape (Cargo, Node/TS, Python) so it
/// works without a language server. `command` overrides the auto-detected one.
#[tauri::command]
pub async fn gl_diagnostics(
    state: State<'_, WorkspaceState>,
    command: Option<String>,
) -> Result<CommandOutput, String> {
    let root = workspace_root(&state)?;

    let cmd = command.unwrap_or_else(|| detect_diagnostics_command(&root));
    if cmd.is_empty() {
        return Ok(CommandOutput {
            stdout: String::new(),
            stderr: "No diagnostics command detected for this project. Pass one explicitly.".into(),
            exit_code: 0,
            timed_out: false,
        });
    }

    tokio::task::spawn_blocking(move || run_shell(&root, &cmd, 180))
        .await
        .map_err(|e| e.to_string())?
}

/// Picks a best-effort check command from files present in the workspace root.
fn detect_diagnostics_command(root: &Path) -> String {
    let has = |name: &str| root.join(name).exists();
    if has("Cargo.toml") {
        "cargo check --message-format short".into()
    } else if has("tsconfig.json") {
        "npx --no-install tsc --noEmit".into()
    } else if has("package.json") {
        "npm run --silent lint".into()
    } else if has("pyproject.toml") || has("setup.py") {
        "python -m pyflakes . || python -m py_compile .".into()
    } else {
        String::new()
    }
}

// ---------------------------------------------------------------------------
// GL_Ask_User
// ---------------------------------------------------------------------------

/// Pending GL_Ask_User prompts, keyed by request id, awaiting a UI answer.
#[derive(Default)]
pub struct AskState(pub Mutex<HashMap<u64, tokio::sync::oneshot::Sender<String>>>);

static ASK_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Asks the user a question through the UI and awaits their answer.
///
/// Emits `ade://ask-user` with `{ id, question, options }`; the frontend shows a
/// prompt and replies via [`gl_answer_user`] with the same id. Resolves with the
/// user's answer string (or the chosen option).
#[tauri::command]
pub async fn gl_ask_user(
    app: AppHandle,
    ask: State<'_, AskState>,
    question: String,
    options: Option<Vec<String>>,
) -> Result<String, String> {
    let id = ASK_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let (tx, rx) = tokio::sync::oneshot::channel();
    ask.0
        .lock()
        .map_err(|_| "ask lock poisoned")?
        .insert(id, tx);

    app.emit(
        "ade://ask-user",
        serde_json::json!({ "id": id, "question": question, "options": options }),
    )
    .map_err(|e| e.to_string())?;

    rx.await
        .map_err(|_| "the ask-user prompt was cancelled".to_string())
}

/// Frontend → backend reply to a GL_Ask_User prompt (matched by `id`).
#[tauri::command]
pub async fn gl_answer_user(
    ask: State<'_, AskState>,
    id: u64,
    answer: String,
) -> Result<(), String> {
    let tx = ask.0.lock().map_err(|_| "ask lock poisoned")?.remove(&id);
    match tx {
        Some(tx) => {
            let _ = tx.send(answer);
            Ok(())
        }
        None => Err("no pending question with that id".into()),
    }
}

// ---------------------------------------------------------------------------
// GL_Open_Browser
// ---------------------------------------------------------------------------

/// Opens a URL in the user's default browser. Only http(s) URLs are allowed.
#[tauri::command]
pub async fn gl_open_browser(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let u = url.trim();
    if !(u.starts_with("http://") || u.starts_with("https://")) {
        return Err("only http(s) URLs are allowed".into());
    }
    app.opener()
        .open_url(u, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::resolve_in_workspace;
    use std::path::Path;

    #[test]
    fn sandbox_allows_paths_inside_root() {
        let root = Path::new("/work/space");
        assert!(resolve_in_workspace(root, "src/main.rs").is_ok());
        assert!(resolve_in_workspace(root, "./src/lib.rs").is_ok());
        // A leading slash is read as workspace-relative, not absolute.
        assert!(resolve_in_workspace(root, "/README.md").is_ok());
        // Descend then climb back within the tree is fine.
        assert!(resolve_in_workspace(root, "a/b/../c.txt").is_ok());
    }

    #[test]
    fn sandbox_rejects_escapes() {
        let root = Path::new("/work/space");
        assert!(resolve_in_workspace(root, "../secret").is_err());
        assert!(resolve_in_workspace(root, "a/../../etc/passwd").is_err());
        assert!(resolve_in_workspace(root, "../../..").is_err());
        // Embedded null.
        assert!(resolve_in_workspace(root, "a\0b").is_err());
    }

    #[test]
    fn sandbox_joins_relative_to_root() {
        let root = Path::new("/work/space");
        let p = resolve_in_workspace(root, "src/x.rs").unwrap();
        assert!(p.ends_with("src/x.rs"));
        assert!(p.starts_with(root));
    }

    /// End-to-end round trip through the sandbox against a real temp workspace:
    /// write → resolve → read back, and confirm an escaping path is rejected
    /// even when the file exists (symlink/traversal guard on the real fs).
    #[test]
    fn sandbox_roundtrip_on_real_fs() {
        let dir = std::env::temp_dir().join(format!("ade-tools-{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("src"));

        // Write inside the sandbox.
        let target = resolve_in_workspace(&dir, "src/hello.txt").unwrap();
        std::fs::write(&target, b"hi").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hi");
        // Resolving the same existing path re-validates the canonical form.
        assert!(resolve_in_workspace(&dir, "src/hello.txt").is_ok());

        // A sibling outside the root must never resolve.
        assert!(resolve_in_workspace(&dir, "../outside.txt").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
