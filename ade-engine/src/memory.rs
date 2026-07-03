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
