# GwenLand ADE

**Agentic Development Environment** — an agentic-first desktop app built with Tauri 2, Svelte 5 (runes), and Vite.

GwenLand ADE ships as a standalone binary, separate from any IDE. It pairs a native Rust engine (`ade-engine`) with a Svelte 5 frontend (`ade-ui`), and keeps a persistent memory layer under `~/.gwenland/ade/memory/` so the agent can learn from past failures and detected preferences.

## Status

`v0.1.0` — early milestone (M1). Core scaffold, workspace picking, and the memory-layer foundation are in place. See the [ROADMAP](ROADMAP.md) for what's next and the [CHANGELOG](CHANGELOG.md) for what's landed.

## Architecture

```
GwenLand ADE/
├── ade-engine/      # Rust + Tauri 2 backend (the binary)
│   └── src/
│       ├── lib.rs        # Tauri builder / entry point
│       ├── commands.rs   # invoke handlers
│       └── memory.rs     # ~/.gwenland/ade/memory/ persistence
└── ade-ui/          # Svelte 5 + Vite frontend
```

### Memory layer

On startup the app ensures `~/.gwenland/ade/memory/` exists and seeds two files if they're absent:

- `failures.md` — failure memory
- `preferences.md` — detected preferences

Existing files are never overwritten. A memory-init failure is non-fatal — the app still runs.

## Dev

Requirements: Rust (stable), Node + pnpm, and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform.

```bash
cd ade-ui && pnpm install      # frontend deps
cd .. && cargo tauri dev       # run the app (from project root)
```

Release build:

```bash
cargo tauri build
```

The release profile is size-tuned (`opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"`, `codegen-units = 1`).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Please also read the [Code of Conduct](CODE_OF_CONDUCT.md).

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
