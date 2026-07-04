# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Unreleased changes are staged as fragments in the [`changelog/`](changelog/)
directory and collected here at release time. See [CONTRIBUTING.md](CONTRIBUTING.md#changelog).

## [Unreleased]

_Nothing yet._

## [0.1.3] — 2026-07-04

Multi-provider support: a typed model registry, secure key storage, and a
model-picker in the composer.

### Added

- Model provider registry (`providers.rs`) — typed, static seed for 14 providers
  with model, context window, and pricing data; three request shapes
  (`Anthropic` / `Gemini` / `OpenAiCompat`) cover all of them (GWEN-464..469).
- New providers: Ziphu (GWEN-464), Groq (GWEN-465), Cohere (GWEN-466),
  Perplexity (GWEN-467), Together AI (GWEN-468).
- Data-driven Settings → API Keys screen (`Settings.svelte`): masked input +
  show/hide per provider, grouped (default three, then alphabetical),
  scrollable. Adding a provider to the registry surfaces it here automatically
  (GWEN-469).
- API-key storage in the OS keychain via the `keyring` crate (`save_api_key` /
  `get_api_key` / `has_api_key`), with env-var fallback.
- From-scratch provider chat clients (`providers.ts`): `fetch` + SSE streaming,
  no HTTP dependency added to the Rust binary.
- Model selector dropdown in the composer (`ModelPicker.svelte`): provider icon
  + model name + `context · $in/$out per 1M` subtitle, grouped by provider,
  opens upward. Replaces the placeholder "ADE Mini" button.
- Vendored provider brand marks (`ProviderIcon.svelte`) — single-path
  `currentColor` SVGs extracted from `@lobehub/icons` at build time; the package
  is not a runtime dependency. Used in both the picker and Settings.

### Changed

- Ziphu (GWEN-464) resolved: "Ziphu" is Zhipu AI, now branded Z.ai — the same
  company as the existing GLM/Z.AI provider. Merged into a single `zai` entry
  with the real `api.z.ai/api/openai/v1` endpoint and current GLM models (5.2,
  5, 4.7, 4.6, 4.5-Air) from Z.ai's official pricing. Dropped the placeholder
  `ziphu`/`glm` duplicates.

## [0.1.2] — 2026-07-03

The human-in-the-loop feedback batch: capture accept/reject/tweak, turn it into
memory, and keep memory bounded.

### Added

- Feedback UI on ADE responses — accept / reject / tweak, attached to the last
  completed message (GWEN-485). Uses `phosphor-svelte` icons already in the
  dependency tree; no new dependencies.
- `record_feedback` Tauri command mapping the user's verdict into a
  `TaskOutcome` and reflecting it into memory; replaces the stubbed post-task
  reflection in `generate()`.
- Heuristic reflection seams `memory::extract_preference` (GWEN-486) and
  `memory::judge_failure` (GWEN-487), ready to swap for model judgment once a
  provider is wired.
- Memory compaction on startup (GWEN-488): once a file exceeds 200 bullets,
  older entries roll up into a `## History (compressed)` line (with count and
  date range) while the newest 50 are kept verbatim. Idempotent and
  best-effort — never fails startup.

### Changed

- Preference extraction collapses multi-line tweaks to one line and caps length
  at 240 chars (GWEN-486).
- Failure judgment skips rejections with no prompt and no output (GWEN-487).
- Reflection de-duplicates entries (ignoring the date stamp) so repeated
  feedback doesn't bloat memory.

## [0.1.1] — 2026-07-03

### Added

- Inject stored memory into the task system prompt via `memory::context_block()`;
  seeded-but-empty files are skipped so no dangling `## Memory` header is emitted
  (GWEN-484).
- Post-task reflection seam `memory::reflect(TaskOutcome)` with `append_memory()`;
  heuristic rules append to `failures.md` / `preferences.md`, non-fatal on write
  error (GWEN-483).
- GitHub Actions pipelines: CI (fmt/clippy/test + frontend build), tagged
  releases via `tauri-action`, and a per-PR changelog-fragment gate.

### Fixed

- Silence two build warnings in `commands.rs` (unused import and unused
  variable).

### Notes

- Local date stamping uses a self-contained civil-from-days helper — no new
  dependency added.

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

[Unreleased]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/JinXSuperSolo/gwenland-ade/releases/tag/v0.1.0
