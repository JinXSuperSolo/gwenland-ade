# GwenLand ADE

GwenLand ADE is an agentic development environment packaged as a standalone
desktop app. It combines a native Rust/Tauri engine with a Svelte frontend so an
AI coding assistant can inspect a selected workspace, edit files, run checks,
open previews, and remember useful feedback over time.

Current version: `0.1.10`  
Status: early alpha / active development

## Highlights

- Standalone desktop shell built with Tauri 2, Rust, Svelte 5, and Vite.
- Provider-backed chat with streaming responses and model/tool-call support.
- Workspace picker plus sandboxed file and command tools scoped to the chosen
  project folder.
- Built-in GL_ agent tools for reading, writing, editing, deleting, grepping,
  globbing, running diagnostics, viewing git diffs, opening browser links, and
  asking the user follow-up questions.
- Provider registry for Anthropic, OpenAI, Google Gemini, DeepSeek, Qwen,
  Z.ai, Kimi, Mistral, xAI/Grok, Groq, Cohere, Perplexity, and Together AI.
- API keys stored in the operating system keychain, with environment-variable
  fallback.
- Persistent local memory under `~/.gwenland/ade/memory/`.
- Inline tool activity, floating terminal output, and on-demand artifact
  previews for HTML, Markdown, Mermaid, and code.

## Project Layout

```text
.
|-- ade-engine/                 # Rust + Tauri backend and desktop binary
|   |-- src/
|   |   |-- commands.rs         # Tauri commands for workspace, keys, memory
|   |   |-- lib.rs              # Tauri builder and invoke handler
|   |   |-- memory.rs           # Markdown memory storage and reflection helpers
|   |   |-- providers.rs        # Static provider/model registry
|   |   |-- tools.rs            # Workspace-sandboxed GL_ tool suite
|   |   `-- window.rs           # Detached preview window support
|   |-- capabilities/
|   `-- tauri.conf.json
|-- ade-ui/                     # Svelte 5 + Vite frontend
|   |-- src/
|   |   |-- components/         # Shell, panes, picker, preview, terminal UI
|   |   |-- features/           # Chat, settings, renderers
|   |   `-- shared/             # Provider, tool, UI, toast, context stores
|   |-- package.json
|   `-- vite.config.ts
|-- Cargo.toml                  # Rust workspace
|-- CHANGELOG.md
|-- CONTRIBUTING.md
|-- ROADMAP.md
`-- LICENSE
```

## Requirements

- Rust stable with `cargo`
- Node.js and `pnpm`
- Tauri 2 system prerequisites for your platform:
  <https://v2.tauri.app/start/prerequisites/>
- Tauri CLI 2 if you run the desktop app through Cargo:

```bash
cargo install tauri-cli --version "^2"
```

## Quick Start

Install frontend dependencies:

```bash
cd ade-ui
pnpm install
```

Run the desktop app:

```bash
cd ../ade-engine
cargo tauri dev
```

The Tauri dev command starts the Vite frontend automatically through
`beforeDevCommand`. Vite runs on `http://localhost:1420`.

## Development Commands

Frontend only:

```bash
cd ade-ui
pnpm dev
pnpm check
pnpm build
```

Rust checks:

```bash
cargo fmt --all
cargo check -p gwenland-ade
cargo test -p gwenland-ade
cargo clippy -p gwenland-ade -- -D warnings
```

Desktop release build:

```bash
cd ade-engine
cargo tauri build
```

The release profile is tuned for smaller binaries with `opt-level = "z"`,
`lto = true`, `strip = true`, `panic = "abort"`, and `codegen-units = 1`.

## Configuration

Open Settings in the app to configure provider API keys. Keys are saved through
the OS keychain:

- Windows Credential Manager on Windows
- Keychain on macOS
- libsecret/Secret Service on Linux

ADE also checks each provider's environment variable before falling back to the
keychain. Common variables include:

- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `GEMINI_API_KEY`
- `DEEPSEEK_API_KEY`
- `KIMI_API_KEY`
- `DASHSCOPE_API_KEY`
- `MISTRAL_API_KEY`
- `XAI_API_KEY`
- `GROQ_API_KEY`
- `COHERE_API_KEY`
- `PERPLEXITY_API_KEY`
- `TOGETHER_API_KEY`
- `ZAI_API_KEY`

## Local Memory

On startup, ADE creates this directory if it does not exist:

```text
~/.gwenland/ade/memory/
```

It seeds two Markdown files:

- `failures.md` records rejected or failed task patterns.
- `preferences.md` records user corrections and detected preferences.

Existing files are never overwritten. Memory initialization and compaction are
best effort, so the app should still open even if memory cannot be written.
Memory can be viewed, edited, exported, and imported from the Settings UI.

## Agent Tooling

The app exposes a provider-neutral GL_ tool suite to the selected model. The
Rust backend resolves all file paths against the selected workspace and rejects
paths that escape it.

Available tools:

- `GL_Read_File`
- `GL_Write_File`
- `GL_Edit_File`
- `GL_Delete_File`
- `GL_List_Dir`
- `GL_Grep`
- `GL_Glob`
- `GL_Git_Diff`
- `GL_Bash`
- `GL_Diagnostics`
- `GL_Ask_User`
- `GL_Open_Browser`
- `GL_OpenPreview`

`GL_Bash` runs in the selected workspace with a timeout and captured output.
`GL_Diagnostics` auto-detects a reasonable check command for Cargo, TypeScript,
Node, or Python projects, and can also be overridden by the agent.

## Architecture Notes

The frontend owns provider streaming with the browser `fetch` API. It translates
the same GL_ tool schema into Anthropic tool use, OpenAI-compatible tool calls,
or Gemini function calls, then routes tool execution back through Tauri invokes.

The backend owns native concerns: workspace selection, file-system sandboxing,
shell execution, keychain storage, memory files, dialogs, OS integration, and
window management.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, checks, changelog practice,
and commit conventions. Please also read the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Security

Security reporting instructions are in [SECURITY.md](SECURITY.md).

## License

GwenLand ADE is licensed under the
[Apache License, Version 2.0](LICENSE).
