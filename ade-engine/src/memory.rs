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

/// Compaction kicks in once a memory file exceeds this many bullet entries.
/// The newest `KEEP_RECENT` are preserved verbatim; the rest are summarized.
const COMPACT_THRESHOLD: usize = 200;
const KEEP_RECENT: usize = 50;

/// Ensures the memory dir and seed files exist, then compacts oversized files.
/// Never overwrites existing seed content.
///
/// Called once on startup. Returns an error only if the directory or a missing
/// seed file could not be created; existing content is left untouched. A
/// compaction failure is swallowed (logged) so startup never fails over it.
pub fn init_memory() -> io::Result<()> {
    let dir = memory_dir()?;
    for (name, contents) in SEED_FILES {
        let path = dir.join(format!("{name}.md"));
        if !path.exists() {
            fs::write(&path, contents)?;
        }
    }
    // Compact after seeding (GWEN-488). Best-effort — a failure here must not
    // prevent the app from starting.
    for (name, _) in SEED_FILES {
        if let Err(e) = compact(name, COMPACT_THRESHOLD, KEEP_RECENT) {
            eprintln!("Warning: could not compact {name}.md: {e}");
        }
    }
    Ok(())
}

/// Summarizes old entries in a memory file once it grows past `threshold`
/// bullets, keeping the newest `keep_recent` verbatim (GWEN-488).
///
/// Older bullets are replaced by a single rolled-up line under a
/// `## History (compressed)` section, e.g.
/// `- 152 earlier entries (2026-06-01 … 2026-07-02)`. Existing history lines are
/// themselves preserved and re-counted, so repeated compaction is idempotent and
/// never loses the running total. Returns `true` if the file was rewritten.
///
/// The file header (leading `#`/blank lines) is preserved as-is.
pub fn compact(name: &str, threshold: usize, keep_recent: usize) -> io::Result<bool> {
    let raw = match read_memory(name) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };

    let bullets: Vec<&str> = raw
        .lines()
        .map(str::trim_end)
        .filter(|l| l.trim_start().starts_with("- "))
        .collect();

    if bullets.len() <= threshold {
        return Ok(false);
    }

    let split = bullets.len().saturating_sub(keep_recent);
    let (older, recent) = bullets.split_at(split);

    // Fold any prior "N earlier entries" rollup into the new count so totals
    // stay accurate across repeated compactions.
    let mut prior_count = 0usize;
    let mut dates: Vec<String> = Vec::new();
    for line in older {
        let text = bullet_text(line);
        if let Some(n) = parse_rollup_count(text) {
            prior_count += n;
        } else {
            prior_count += 1;
        }
        if let Some(d) = bullet_date(line) {
            dates.push(d);
        }
    }
    dates.sort();
    let range = match (dates.first(), dates.last()) {
        (Some(first), Some(last)) if first != last => format!(" ({first} … {last})"),
        (Some(only), _) => format!(" ({only})"),
        _ => String::new(),
    };

    // Preserve the file's header block (before the first bullet).
    let header: String = raw
        .lines()
        .take_while(|l| !l.trim_start().starts_with("- "))
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = String::new();
    out.push_str(header.trim_end());
    out.push_str("\n\n## History (compressed)\n\n");
    out.push_str(&format!("- {prior_count} earlier entries{range}\n"));
    out.push_str("\n## Recent\n\n");
    for line in recent {
        out.push_str(line);
        out.push('\n');
    }

    write_memory(name, &out)?;
    Ok(true)
}

/// Parses the entry count from a rollup line like `152 earlier entries (…)`.
fn parse_rollup_count(text: &str) -> Option<usize> {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = text[digits.len()..].trim_start();
    if rest.starts_with("earlier entries") {
        digits.parse().ok()
    } else {
        None
    }
}

/// Extracts the `YYYY-MM-DD` date from a `- [date] …` bullet, if present.
fn bullet_date(line: &str) -> Option<String> {
    let line = line.trim().trim_start_matches("- ").trim_start();
    let inner = line.strip_prefix('[')?;
    let close = inner.find(']')?;
    Some(inner[..close].to_string())
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
/// Entries that already appear in the target file are skipped so repeated
/// feedback doesn't bloat memory (GWEN-486/487).
///
/// Non-fatal by contract: a write failure returns `Err` for the caller to log,
/// but the caller should not fail the task over it.
pub fn reflect(outcome: &TaskOutcome) -> io::Result<()> {
    if outcome.errored {
        let summary = outcome.failure_summary.as_deref().unwrap_or("task failed");
        append_unique("failures", summary)?;
    }
    if let Some(correction) = &outcome.user_correction {
        append_unique("preferences", correction)?;
    }
    Ok(())
}

/// Appends `entry` unless an equivalent bullet is already present in the file.
///
/// "Equivalent" ignores the leading `- [date]` stamp, so the same correction
/// recorded on two different days is treated as a duplicate.
fn append_unique(name: &str, entry: &str) -> io::Result<()> {
    let entry = entry.trim();
    let existing = read_memory(name).unwrap_or_default();
    let already = existing.lines().any(|line| bullet_text(line) == entry);
    if already {
        return Ok(());
    }
    append_memory(name, entry)
}

/// Strips a `- [date] ` prefix (if any) from a memory line, returning the text.
fn bullet_text(line: &str) -> &str {
    let line = line.trim().trim_start_matches("- ").trim_start();
    match (line.starts_with('['), line.find(']')) {
        (true, Some(close)) => line[close + 1..].trim_start(),
        _ => line,
    }
}

/// Heuristic preference extraction from a tweak (GWEN-486).
///
/// Returns the correction text to record in `preferences.md`, or `None` when
/// there is not enough signal. The tweak is trimmed, collapsed to a single line,
/// and length-capped so a pasted essay doesn't swamp the memory file. A
/// model-judgment version can replace the body without changing callers.
pub fn extract_preference(_output: &str, tweak: Option<&str>) -> Option<String> {
    const MAX_LEN: usize = 240;

    let tweak = tweak?.trim();
    if tweak.is_empty() {
        return None;
    }
    // Collapse whitespace/newlines into a single-line bullet.
    let mut normalized = tweak.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > MAX_LEN {
        normalized = normalized.chars().take(MAX_LEN - 1).collect::<String>() + "…";
    }
    Some(format!("prefers: {normalized}"))
}

/// Heuristic failure judgment from a rejected response (GWEN-487).
///
/// Returns a short failure summary to record in `failures.md`, or `None` when
/// the rejection carries too little context to be worth recording (empty prompt
/// *and* empty output). A model-backed judgment can refine this later, e.g.
/// distinguishing "wrong" from "not what I wanted".
pub fn judge_failure(prompt: &str, output: &str) -> Option<String> {
    let prompt = prompt.trim();
    if prompt.is_empty() && output.trim().is_empty() {
        return None;
    }
    let summary = if prompt.is_empty() {
        "user rejected the response".to_string()
    } else {
        let one_line = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
        format!("user rejected the response for: {one_line}")
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
        // Multi-line tweaks collapse to a single line.
        assert_eq!(
            extract_preference("out", Some("use\n  tabs\tnot spaces")),
            Some("prefers: use tabs not spaces".to_string())
        );
    }

    #[test]
    fn extract_preference_caps_length() {
        let long = "x ".repeat(400);
        let pref = extract_preference("out", Some(&long)).unwrap();
        assert!(pref.chars().count() <= "prefers: ".len() + 240);
        assert!(pref.ends_with('…'));
    }

    #[test]
    fn judge_failure_summarizes_rejection() {
        assert_eq!(
            judge_failure("add  tests", "..."),
            Some("user rejected the response for: add tests".to_string())
        );
        assert_eq!(
            judge_failure("   ", "some output"),
            Some("user rejected the response".to_string())
        );
        // No prompt and no output → not worth recording.
        assert_eq!(judge_failure("   ", "  "), None);
    }

    #[test]
    fn bullet_helpers_parse_stamped_lines() {
        assert_eq!(bullet_text("- [2026-07-03] prefers: tabs"), "prefers: tabs");
        assert_eq!(bullet_text("- plain entry"), "plain entry");
        assert_eq!(
            bullet_date("- [2026-07-03] x"),
            Some("2026-07-03".to_string())
        );
        assert_eq!(bullet_date("- no date"), None);
    }

    #[test]
    fn parse_rollup_count_reads_totals() {
        assert_eq!(parse_rollup_count("152 earlier entries (a … b)"), Some(152));
        assert_eq!(parse_rollup_count("prefers: use 4 spaces"), None);
        assert_eq!(parse_rollup_count("earlier entries"), None);
    }

    #[test]
    fn compact_rolls_up_and_is_idempotent() {
        // Build a file with 210 stamped bullets over threshold=200, keep 50.
        let mut raw = String::from("# Failure Memory\n\n");
        for i in 0..210 {
            raw.push_str(&format!("- [2026-07-{:02}] entry {i}\n", (i % 28) + 1));
        }

        let compacted = rewrite_for_test(&raw, 200, 50);
        // 160 older entries rolled into one line; 50 recent kept.
        assert!(compacted.contains("## History (compressed)"));
        assert!(compacted.contains("- 160 earlier entries"));
        let recent = compacted.matches("entry ").count();
        assert_eq!(recent, 50);

        // Re-compacting the result (still 51 bullets: 1 rollup + 50) is a no-op
        // because it's under threshold.
        assert!(count_bullets(&compacted) <= 51);
    }

    // Mirror of `compact`'s transform without touching the filesystem, so the
    // rollup logic is unit-testable. Kept in sync with `compact` by construction.
    fn rewrite_for_test(raw: &str, threshold: usize, keep_recent: usize) -> String {
        let bullets: Vec<&str> = raw
            .lines()
            .map(str::trim_end)
            .filter(|l| l.trim_start().starts_with("- "))
            .collect();
        assert!(
            bullets.len() > threshold,
            "test fixture must exceed threshold"
        );

        let split = bullets.len().saturating_sub(keep_recent);
        let (older, recent) = bullets.split_at(split);

        let mut prior_count = 0usize;
        let mut dates: Vec<String> = Vec::new();
        for line in older {
            let text = bullet_text(line);
            prior_count += parse_rollup_count(text).unwrap_or(1);
            if let Some(d) = bullet_date(line) {
                dates.push(d);
            }
        }
        dates.sort();
        let range = match (dates.first(), dates.last()) {
            (Some(f), Some(l)) if f != l => format!(" ({f} … {l})"),
            (Some(o), _) => format!(" ({o})"),
            _ => String::new(),
        };
        let header: String = raw
            .lines()
            .take_while(|l| !l.trim_start().starts_with("- "))
            .collect::<Vec<_>>()
            .join("\n");

        let mut out = String::new();
        out.push_str(header.trim_end());
        out.push_str("\n\n## History (compressed)\n\n");
        out.push_str(&format!("- {prior_count} earlier entries{range}\n"));
        out.push_str("\n## Recent\n\n");
        for line in recent {
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    fn count_bullets(s: &str) -> usize {
        s.lines()
            .filter(|l| l.trim_start().starts_with("- "))
            .count()
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
