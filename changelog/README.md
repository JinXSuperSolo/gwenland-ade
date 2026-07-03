# Changelog fragments

Unreleased changes live here as small fragment files, one per change, instead of
editing the top-level [`CHANGELOG.md`](../CHANGELOG.md) directly. This keeps
parallel PRs from conflicting on the same lines.

## Adding a fragment

Create a file named `<type>-<slug>.md`, where `<type>` is one of `added`,
`changed`, `deprecated`, `removed`, `fixed`, or `security`:

```
changelog/added-memory-seed.md
```

Contents use a Keep a Changelog heading:

```md
### Added
- Auto-create `failures.md` and `preferences.md` on first run (GWEN-482).
```

## At release time

Fragments in this directory are collected into `CHANGELOG.md` under the new
version, grouped by heading, and then removed. Only this `README.md` and the
`.gitkeep` remain.
