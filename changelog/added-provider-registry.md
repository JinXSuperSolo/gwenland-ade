### Added
- Model provider registry (`providers.rs`) — typed, static seed for 14 providers with model, context window, and pricing data; three request shapes (`Anthropic` / `Gemini` / `OpenAiCompat`) cover all of them (GWEN-464..469).
- New providers: Ziphu (GWEN-464), Groq (GWEN-465), Cohere (GWEN-466), Perplexity (GWEN-467), Together AI (GWEN-468).
- Data-driven Settings → API Keys screen (`Settings.svelte`): masked input + show/hide per provider, grouped (default three, then alphabetical), scrollable. Adding a provider to the registry surfaces it here automatically (GWEN-469).
- API-key storage in the OS keychain via the `keyring` crate (`save_api_key` / `get_api_key` / `has_api_key`), with env-var fallback.
- From-scratch provider chat clients (`providers.ts`): `fetch` + SSE streaming, no HTTP dependency added to the Rust binary.
