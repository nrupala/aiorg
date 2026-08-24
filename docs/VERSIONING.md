# VERSIONING — AIORG's application of the App Versioning Standard

Adopted from Nrupal's App Versioning Standard (`nrupala/milo-skill-map`,
reference implementations: research-analyst rollout PRs). Non-JS repo rule
applies: a top-level **`VERSION`** file is the single source of truth.

## Principle
One version, one source of truth, stamped onto every surface. Never hand-edit
a version per file; never ship whose internal version does not match the tag.

## Source of truth & releases
- Canonical: `VERSION` (SemVer `X.Y.Z`) — also mirrored into
  `engine/Cargo.toml [workspace.package] version` by the bump script so
  `cargo metadata` agrees.
- A release == a git tag `v<X.Y.Z>`. The tag MUST equal `VERSION`; CI fails
  otherwise (release.yml "Version guard").
- `MINOR` and `PATCH` stay `< 100`.

## Every surface derives from that one version

| Surface | Mechanism |
|---|---|
| Binary | `aiorg --version`, banner in every command — stamped at compile time by `engine/cli/build.rs` reading `../../VERSION` |
| Doctor/status output | same compiled constant |
| SSE `/status`, MCP `serverInfo.version` | same constant (wired at M6) |
| CHANGELOG | `CHANGELOG.md`, Keep a Changelog; bump promotes `[Unreleased]` |
| GitHub Release notes | `generate_release_notes: true` in release.yml |

## Two moving parts
1. **Author time** — `node scripts/bump-version.mjs <X.Y.Z>`: writes `VERSION`,
   syncs Cargo workspace version, promotes CHANGELOG `[Unreleased]` →
   `[X.Y.Z] - YYYY-MM-DD`, opens a fresh `[Unreleased]`.
2. **Build time** — `build.rs` stamps the binary; release.yml guards
   tag==VERSION, builds on windows-latest, attaches the exe, generates notes.

## Release flow
```
edit + PR (never direct-to-main) → merge
node scripts/bump-version.mjs 0.2.0   # commit "chore(release): 0.2.0"
git tag v0.2.0 && git push --follow-tags
# CI: guard passes → build → GitHub Release vX.Y.Z with auto notes
```

## Files
`VERSION` · `CHANGELOG.md` · `scripts/bump-version.mjs` ·
`.github/workflows/release.yml` · mirror `deploy/github-actions/release.yml`
(paste-via-web-editor fallback when a token lacks `workflow` scope).
