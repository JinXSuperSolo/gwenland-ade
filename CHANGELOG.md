# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- API Keys screen typography pass (GWEN-502).
- Model picker padding + bento cards (GWEN-503).
- Floating preview window + terminal (GWEN-504).

### Planned

Coming soon — tracked, not yet shipped:

- **Plus (`+`) button context menu** — a rich menu on the composer's add button:
  - Import from GitHub
  - Create from Figma (Premium)
  - Upload from computer
  - Generate Images (toggle)
  - Skills →
  - Design Systems →
  - Folder → (search + create)
  - Instructions → (search + select + create)
  - MCPs → (suggested list + custom)
  - Full Permissions → (Ask / Auto / Full)
- **Agent state UX** — shimmer while thinking + completion toast.
- **Error filtering** — surface only actionable errors in the thread.

## [0.1.6] — 2026-07-04

M5 (batch 2) — the agent comes alive: a real provider-driven tool loop, the
GL_ tool suite, and a Claude.ai-style chat + on-demand artifact preview.

### Added

- GL_ agent tool suite (`tools.rs`): 12 workspace-sandboxed tools — `GL_Read_File`,
  `GL_Write_File`, `GL_Edit_File`, `GL_Delete_File`, `GL_List_Dir`, `GL_Grep`,
  `GL_Glob`, `GL_Git_Diff`, `GL_Bash`, `GL_Diagnostics`, `GL_Ask_User`, and
  `GL_Open_Browser`. Every path-taking tool is resolved against the workspace
  root and rejects escapes (`..`, absolute paths, symlink traversal); shell tools
  dispatch to the OS-native tooling (PowerShell on Windows, `grep`/`find`/`sh`
  elsewhere). `GL_Bash` runs with a timeout and captured output.
- Real agent tool-loop (`conversation.svelte.ts` + `providers.ts`): `send()` now
  streams from the selected provider with the GL_ tools registered, runs any tool
  calls via `callTool`, feeds results back, and repeats until the model answers.
  Tool-use is implemented for all three provider shapes — Anthropic `tool_use`,
  OpenAI `tool_calls`, and Gemini `functionCall`.
- Inline tool activity + GL_Ask_User (`ToolCall.svelte`, `AskPrompt.svelte`):
  Claude.ai-style collapsible tool rows (running → done/error) and an inline
  question prompt with clickable options or a free-text reply, backed by the
  `gl_ask_user` / `gl_answer_user` request/response channel.
- On-demand artifact preview: a new `GL_OpenPreview` tool renders an artifact
  (HTML / Markdown / Mermaid / code) in a side pane. HTML runs in a sandboxed
  iframe; Markdown and Mermaid go through the from-scratch renderers. The pane is
  hidden by default and opens (with a slide/fade transition) when the agent shows
  a result. `tauri-plugin-opener` powers `GL_Open_Browser`.
- Provider-neutral tool schema + dispatcher (`shared/tools.ts`): typed `GL.*`
  wrappers, a `GL_TOOLS` JSON-Schema array translatable to Anthropic/OpenAI/Gemini
  tool formats, and a `callTool(name, args)` router.

### Changed

- Chat is now the main view (`ChatView.svelte`): the conversation thread fills
  the primary surface with the composer docked at the bottom (Claude.ai layout),
  extracted into a standalone `ConversationView.svelte` shared by the in-split and
  detached surfaces. The preview pane became a separate, hidden-by-default artifact
  panel that opens only on `GL_OpenPreview` — fixing the cramped/right-clipped
  output layout.
- Opening/closing the preview pane animates (grid-track + slide/fade transition);
  the resize handle skips the animation while dragging.

### Fixed

- The conversation was only rendered inside the preview pane, so hiding the pane
  hid the chat; the chat now lives in its own main view.
- Robust API-key check before streaming: a keychain read error no longer surfaces
  as a raw error, and an empty key gives a clear "set up your token" message.

## [0.1.5] — 2026-07-04

M5 (batch 1) — a capability-aware model picker plus three from-scratch content
renderers (Markdown, KaTeX, Mermaid). Also an `ade-ui/src` reorganization into
`components/`, `features/`, and `shared/`.

### Added

- Per-model reasoning capability (M5): a `Reasoning` enum on every registry
  model (`providers.rs`) — `none`, `effort` (Anthropic adaptive low/med/high/max),
  `budget_tokens` (legacy toggle), `always_on`, `reasoning_effort` (OpenAI
  o-series low/med/high), `thinking_level` (Gemini 3.x), `thinking_budget`
  (Gemini 2.5 Flash / GLM toggle). Surfaced through `list_providers` as derived
  UI flags (`reasoningLevels` / `reasoningToggle` / `reasoningMax`).
- Capability-aware model picker (`ModelPicker.svelte`): the effort/thinking UI
  now renders per selected model — a level selector where the model supports one
  (with `max` only where valid), an on/off toggle for budget-style thinking, an
  "always on" note for unconditional reasoners, and nothing for plain chat
  models. `reasoningParams()` (`providers.ts`) maps the chosen level/toggle to
  the correct wire parameter per provider kind.
- Markdown renderer from scratch (`markdown.ts` + `Markdown.svelte` /
  `MarkdownInline.svelte`): headings, nested ordered/unordered lists, GFM tables
  with alignment, fenced code, blockquotes, thematic breaks, and full inline
  (bold/italic/strike/code/links/images/autolinks/hard breaks). No dependencies.
- KaTeX-style math renderer from scratch (`katex.ts` + `Katex.svelte`):
  fractions, roots, sub/superscripts, Greek + symbol macros, named operators,
  accents, `\left`/`\right` delimiters, and `$…$` / `$$…$$` spans. Degrades to
  the escaped source on any parse failure.
- Mermaid renderer from scratch (`mermaid.ts` + `Mermaid.svelte`): `flowchart`
  /`graph` (layered layout, all node shapes, edge labels, TD/LR direction) and
  `sequenceDiagram` (lifelines, arrows, self-messages), emitting themed SVG.
  Unsupported diagram types degrade to a note with the source.
- ADE responses now render through the Markdown pipeline in the output thread
  (`Output.svelte`), routing `mermaid` code fences and math spans to their
  renderers.
- Kimi frontier model lineup (K2 series): `kimi-k2-6`, `kimi-k2-6-thinking`,
  `kimi-k2-5`, `kimi-k2-thinking`, `kimi-k2-0905-preview`, `kimi-k2-turbo-preview`,
  `kimi-k2-instruct`, and `kimi-latest`.

### Changed

- Reorganized `ade-ui/src` from flat files into `components/`,
  `features/{chat,renderers,settings}/`, and `shared/` for clearer module
  boundaries.
- Kimi provider corrected: the product is **Kimi** (Moonshot AI is the company),
  so the display name is now "Kimi", the key env is `KIMI_API_KEY`, and the
  endpoint is `api.moonshot.ai`. Model ids moved to the current `kimi-k2-*` line.
- Corrected brand icons (`ProviderIcon.svelte`): the official Kimi "K" mark
  (`#1783FF`) and Z.ai / Zhipu color (`#3859FF`).

### Fixed

- Added the missing `zai` (Z.ai / GLM) provider and `gemini-2.5-flash` model,
  fixing a failing `spec_providers_all_present` registry test.
- Fixed an infinite loop in the nested-list Markdown parser (the item-collection
  loop failed to advance on deeper-indented lines).

## [0.1.4] — 2026-07-04

M4 — ADE identity & polish: a split-pane workspace with a detachable preview,
first-time onboarding, and a Settings hub that houses the memory viewer and
export/import.

### Added

- Split-pane workspace (GWEN-489): resizable composer/preview layout
  (`SplitPane.svelte`) with a drag handle, min widths, and a ratio persisted to
  `localStorage`. Conversation state moved to a shared runes module
  (`conversation.svelte.ts`) so both panes read one source of truth.
- Detachable preview: `open_preview_window` (`window.rs`) pops the output into a
  second Tauri window (`PreviewWindow.svelte`, loaded via `index.html?preview`);
  closing it re-attaches automatically via an `ade://preview-closed` event.
- Thin status bar (`StatusBar.svelte`, 28px) showing workspace name, a Memory
  shortcut, and the active model.
- First-time onboarding (GWEN-490): a guided composer empty-state
  (`OnboardingOverlay.svelte`) shown when there's no memory yet, detected via a
  new `has_memory` command. Prompts for a workspace on first interaction and
  surfaces a one-time, auto-dismissing "detach preview" hint after the first
  generate.
- `get_username` command for the composer greeting (previously called by the UI
  but never registered).
- Settings hub with a bento-style launcher (`Settings.svelte`): a hero
  "Set up Token" card with an animated fill-and-delete API-key mockup, plus
  Memory and About panels with icon list-rows. Cards open each section full-view
  with a back arrow.
- In-app memory viewer/editor (GWEN-491), now a Settings section
  (`SettingsMemory.svelte`): tabs for `failures.md` / `preferences.md`, an
  editable textarea, and Save, backed by `read_memory_file` / `write_memory_file`
  commands. Reachable via the gear, `Ctrl+M`, or the status-bar Memory button.
- Memory export/import (GWEN-492): `export_memory` / `import_memory` commands zip
  and restore the memory directory through native dialogs, with an overwrite
  confirmation on import. Adds the `zip` crate (deflate-only, size-tuned).

### Changed

- The preview pane is now hidden by default (full-width centered composer) and
  auto-reveals once there's output to show. A titlebar toggle (top-right, by the
  window controls) shows/hides it manually.
- Memory moved out of a slide-in panel into the Settings hub; the standalone
  `MemoryPanel.svelte` was removed. `Ctrl+M` and the status-bar Memory button now
  deep-link to Settings → Memory.

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

[Unreleased]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/JinXSuperSolo/gwenland-ade/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/JinXSuperSolo/gwenland-ade/releases/tag/v0.1.0
