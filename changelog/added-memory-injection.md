### Added
- Inject stored memory into the task system prompt via `memory::context_block()`; seeded-but-empty files are skipped (GWEN-484).
- Post-task reflection seam `memory::reflect(TaskOutcome)` with `append_memory()`; heuristic rules append to `failures.md` / `preferences.md`, non-fatal on write error (GWEN-483).
