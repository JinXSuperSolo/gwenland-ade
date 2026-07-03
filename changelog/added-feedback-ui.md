### Added
- Feedback UI on ADE responses — accept / reject / tweak, attached to the last completed message (GWEN-485).
- `record_feedback` Tauri command mapping the user's verdict into a `TaskOutcome` and reflecting it into memory; replaces the stubbed post-task reflection in `generate()`.
- Heuristic seams `memory::extract_preference` (GWEN-486) and `memory::judge_failure` (GWEN-487), ready to swap for model judgment once a provider is wired.
