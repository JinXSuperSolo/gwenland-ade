# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Unreleased changes are staged as fragments in the [`changelog/`](changelog/)
directory and collected here at release time. See [CONTRIBUTING.md](CONTRIBUTING.md#changelog).

## [Unreleased]

_Nothing yet._

## [0.1.0] — 2026-07-03

Initial milestone (M1) — foundation.

### Added

- Standalone Tauri 2 + Svelte 5 (runes) desktop app scaffold, built as a binary
  separate from any IDE.
- Workspace picker commands (`pick_workspace`, `get_workspace`).
- Memory-layer foundation (GWEN-481): `~/.gwenland/ade/memory/` is created on
  startup.
- Auto-created seed files (GWEN-482): `failures.md` and `preferences.md` are
  written on first run and never overwritten if they already exist.
- Non-fatal memory init: a memory-layer failure logs a warning and the app
  continues to run.

### Notes

- Release profile is size-tuned (`opt-level = "z"`, `lto`, `strip`,
  `panic = "abort"`, `codegen-units = 1`).

[Unreleased]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/JinXSuperSolo/gwenland-ade/releases/tag/v0.1.0
