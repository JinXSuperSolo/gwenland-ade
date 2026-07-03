### Added
- Model selector dropdown in the composer (`ModelPicker.svelte`): provider icon + model name + `context · $in/$out per 1M` subtitle, grouped by provider, opens upward. Replaces the placeholder "ADE Mini" button.
- Vendored provider brand marks (`ProviderIcon.svelte`) — single-path `currentColor` SVGs extracted from `@lobehub/icons` at build time; the package is not a runtime dependency. Used in both the picker and Settings.

### Changed
- Ziphu (GWEN-464) resolved: "Ziphu" is Zhipu AI, now branded Z.ai — the same company as the existing GLM/Z.AI provider. Merged into a single `zai` entry with the real `api.z.ai/api/openai/v1` endpoint and current GLM models (5.2, 5, 4.7, 4.6, 4.5-Air) from Z.ai's official pricing. Dropped the placeholder `ziphu`/`glm` duplicates.
