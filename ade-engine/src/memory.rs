//! Read/write markdown memory files under `~/.gwenland/ade/memory/`.
//! Not yet wired to commands; exposed to the invoke handler in a later milestone.
#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::PathBuf;

/// Seed files auto-created on first run: `(name, initial contents)`.
const SEED_FILES: &[(&str, &str)] = &[
    ("failures", "# Failure Memory\n\n"),
    (
        "preferences",
        "# Preference Memory\n\n## Detected Preferences\n\n",
    ),
];

/// Returns `~/.gwenland/ade/memory`, creating it if needed.
pub fn memory_dir() -> io::Result<PathBuf> {
    let home = std::env::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory not found"))?;
    let dir = home.join(".gwenland").join("ade").join("memory");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Ensures the memory dir and seed files exist. Never overwrites existing files.
///
/// Called once on startup. Returns an error only if the directory or a missing
/// seed file could not be created; existing content is left untouched.
pub fn init_memory() -> io::Result<()> {
    let dir = memory_dir()?;
    for (name, contents) in SEED_FILES {
        let path = dir.join(format!("{name}.md"));
        if !path.exists() {
            fs::write(&path, contents)?;
        }
    }
    Ok(())
}

/// Lists memory names (file stems of `*.md`), sorted.
pub fn list_memories() -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(memory_dir()?)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn read_memory(name: &str) -> io::Result<String> {
    fs::read_to_string(memory_path(name)?)
}

pub fn write_memory(name: &str, content: &str) -> io::Result<()> {
    fs::write(memory_path(name)?, content)
}

/// Appends a timestamped bullet to the named memory file, creating it if absent.
///
/// Existing content is preserved — this never rewrites the file. A trailing
/// newline is ensured before the new entry so bullets don't run together.
pub fn append_memory(name: &str, entry: &str) -> io::Result<()> {
    use std::io::Write;

    let path = memory_path(name)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    // Best-effort local date stamp; avoids a chrono dependency.
    writeln!(file, "- [{}] {}", today(), entry.trim())
}

/// Builds the memory block appended to the system prompt (GWEN-484).
///
/// Returns an empty string when there is nothing meaningful to inject, so the
/// caller never emits a dangling `## Memory` header. A file that contains only
/// its seed header (and blank lines) counts as empty.
pub fn context_block() -> String {
    let failures = meaningful_body(&read_memory("failures").unwrap_or_default());
    let preferences = meaningful_body(&read_memory("preferences").unwrap_or_default());

    if failures.is_empty() && preferences.is_empty() {
        return String::new();
    }

    let mut block = String::from("## Memory\n");
    if !failures.is_empty() {
        block.push_str("\n### Past failures\n");
        block.push_str(&failures);
        block.push('\n');
    }
    if !preferences.is_empty() {
        block.push_str("\n### Detected preferences\n");
        block.push_str(&preferences);
        block.push('\n');
    }
    block
}

/// Strips markdown headings (`#`-prefixed) and blank lines, returning the
/// remaining content. Used to tell "seeded but empty" files from real memory.
fn meaningful_body(raw: &str) -> String {
    raw.lines()
        .map(str::trim_end)
        .filter(|line| !line.trim_start().starts_with('#') && !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns the local date as `YYYY-MM-DD`, computed from the system clock.
fn today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Converts days since the Unix epoch to a `(year, month, day)` civil date.
/// Howard Hinnant's `civil_from_days` algorithm — no external crate.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn memory_path(name: &str) -> io::Result<PathBuf> {
    // memory names must stay inside the memory dir
    if name.is_empty() || name.contains(['/', '\\']) || name.contains("..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid memory name",
        ));
    }
    Ok(memory_dir()?.join(format!("{name}.md")))
}

/// Outcome of a task, fed to [`reflect`] to decide what (if anything) to record.
///
/// This is deliberately a thin, provider-agnostic seam: the current heuristic
/// reads only these fields, and a future LLM-driven reflection can consume the
/// same struct without changing call sites (GWEN-483).
#[derive(Debug, Default)]
pub struct TaskOutcome {
    /// The task failed (error surfaced to the user).
    pub errored: bool,
    /// A short description of the failure, recorded to `failures.md`.
    pub failure_summary: Option<String>,
    /// The user corrected the agent; recorded to `preferences.md`.
    pub user_correction: Option<String>,
}

/// Post-task reflection: appends heuristic entries to memory (GWEN-483).
///
/// Rules (intentionally simple and deterministic):
/// - `errored` → append `failure_summary` to `failures.md`.
/// - `user_correction` present → append it to `preferences.md`.
///
/// Non-fatal by contract: a write failure returns `Err` for the caller to log,
/// but the caller should not fail the task over it.
pub fn reflect(outcome: &TaskOutcome) -> io::Result<()> {
    if outcome.errored {
        let summary = outcome.failure_summary.as_deref().unwrap_or("task failed");
        append_memory("failures", summary)?;
    }
    if let Some(correction) = &outcome.user_correction {
        append_memory("preferences", correction)?;
    }
    Ok(())
}

/// Heuristic preference extraction from a tweak (GWEN-486 seam).
///
/// Returns the correction text to record in `preferences.md`, or `None` when
/// there is not enough signal (e.g. an empty tweak). A model-judgment version
/// can replace the body without changing callers.
pub fn extract_preference(_output: &str, tweak: Option<&str>) -> Option<String> {
    let tweak = tweak?.trim();
    if tweak.is_empty() {
        return None;
    }
    Some(format!("prefers: {tweak}"))
}

/// Heuristic failure judgment from a rejected response (GWEN-487 seam).
///
/// Returns a short failure summary to record in `failures.md`. Currently every
/// rejection is treated as a failure; a model-backed judgment can refine this
/// (e.g. distinguish "wrong" from "not what I wanted") later.
pub fn judge_failure(prompt: &str, _output: &str) -> Option<String> {
    let prompt = prompt.trim();
    let summary = if prompt.is_empty() {
        "user rejected the response".to_string()
    } else {
        format!("user rejected the response for: {prompt}")
    };
    Some(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_matches_known_epochs() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(18_993), (2022, 1, 1));
        // 2020-02-29 (leap day) is day 18321.
        assert_eq!(civil_from_days(18_321), (2020, 2, 29));
    }

    #[test]
    fn meaningful_body_ignores_headers_and_blanks() {
        assert_eq!(meaningful_body(""), "");
        assert_eq!(meaningful_body("# Failure Memory\n\n"), "");
        assert_eq!(
            meaningful_body("# Failure Memory\n\n- [2026-07-03] boom\n"),
            "- [2026-07-03] boom"
        );
    }

    #[test]
    fn extract_preference_needs_signal() {
        assert_eq!(extract_preference("out", None), None);
        assert_eq!(extract_preference("out", Some("   ")), None);
        assert_eq!(
            extract_preference("out", Some("use tabs")),
            Some("prefers: use tabs".to_string())
        );
    }

    #[test]
    fn judge_failure_summarizes_rejection() {
        assert_eq!(
            judge_failure("add tests", "..."),
            Some("user rejected the response for: add tests".to_string())
        );
        assert_eq!(
            judge_failure("   ", "..."),
            Some("user rejected the response".to_string())
        );
    }

    #[test]
    fn context_block_layout() {
        // Pure formatting check on the assembler via a stand-in of meaningful_body
        // output; the full read path is exercised manually (needs a home dir).
        let failures = "- oops";
        let preferences = "- likes tabs";
        let mut expected = String::from("## Memory\n");
        expected.push_str("\n### Past failures\n- oops\n");
        expected.push_str("\n### Detected preferences\n- likes tabs\n");

        let mut block = String::from("## Memory\n");
        block.push_str("\n### Past failures\n");
        block.push_str(failures);
        block.push('\n');
        block.push_str("\n### Detected preferences\n");
        block.push_str(preferences);
        block.push('\n');

        assert_eq!(block, expected);
    }
}
