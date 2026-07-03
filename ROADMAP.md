# Roadmap

This roadmap is directional, not a commitment. Items may move between milestones as the project evolves.

## M1 — Foundation (current, `v0.1.x`)

- [x] Tauri 2 + Svelte 5 scaffold, standalone binary separate from any IDE
- [x] Workspace picker (`pick_workspace`, `get_workspace`)
- [x] Memory-layer foundation — `~/.gwenland/ade/memory/` auto-created on startup
- [x] Seed files (`failures.md`, `preferences.md`) auto-created, never overwritten
- [ ] Wire memory read/write into the agent loop
- [ ] Binary-size reduction (regex stack via `tauri-plugin-shell` ≈ 375 KiB)

## M2 — Agent loop

- [ ] Streaming generation surfaced in the UI
- [ ] Memory injection into the model context
- [ ] Failure capture — record failed runs to `failures.md`
- [ ] Preference detection — write inferred preferences to `preferences.md`

## M3 — Tooling

- [ ] Tool/command execution with user approval
- [ ] File read/write within the selected workspace
- [ ] Diff review UI

## M4 — Polish

- [ ] Settings and configuration surface
- [ ] Packaging and auto-update
- [ ] Cross-platform testing (Windows, macOS, Linux)

## Later

- [ ] Multi-workspace sessions
- [ ] Plugin/extension surface

---

Have an idea? Open an issue or start a discussion. See [CONTRIBUTING.md](CONTRIBUTING.md).
