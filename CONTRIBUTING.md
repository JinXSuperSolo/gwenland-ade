# Contributing to GwenLand ADE

Thanks for your interest in contributing. This document covers how to get set up,
the conventions we follow, and how to submit changes.

By participating, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting started

Requirements:

- Rust (stable) with `cargo`
- Node + [pnpm](https://pnpm.io/)
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

```bash
cd ade-ui && pnpm install
cd .. && cargo tauri dev
```

## Project layout

- `ade-engine/` — Rust + Tauri 2 backend (the binary)
- `ade-ui/` — Svelte 5 (runes) + Vite frontend
- `changelog/` — unreleased changelog fragments (see below)

## Before you open a PR

Run these locally and make sure they pass:

```bash
# Rust
cargo fmt --all
cargo clippy -p gwenland-ade -- -D warnings
cargo check -p gwenland-ade

# Frontend
cd ade-ui && pnpm check
```

Keep changes focused. Match the surrounding code's style, naming, and comment
density rather than introducing a new one.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add streaming generation to the UI
fix: never overwrite existing memory files
docs: expand the memory-layer section in the README
chore: bump tauri to 2.11.5
```

Reference the relevant ticket (e.g. `GWEN-481`) in the body when applicable.

## Changelog

We keep unreleased changes as fragments in the [`changelog/`](changelog/)
directory instead of editing `CHANGELOG.md` directly (this avoids merge
conflicts). Add a short file per change, named `<type>-<slug>.md`, where
`<type>` is one of `added`, `changed`, `deprecated`, `removed`, `fixed`, or
`security` — for example `changelog/added-memory-seed.md`:

```md
### Added
- Auto-create `failures.md` and `preferences.md` on first run (GWEN-482).
```

At release time these fragments are collected into [CHANGELOG.md](CHANGELOG.md)
under the new version and the directory is cleared. See
[`changelog/README.md`](changelog/README.md) for details.

## Pull requests

1. Fork and create a topic branch (`feat/…`, `fix/…`, `docs/…`).
2. Make your change, add a changelog fragment, and run the checks above.
3. Open a PR describing **what** changed and **why**. Link related issues.
4. Be responsive to review feedback.

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License, Version 2.0](LICENSE), the same license as the project.
