# Contributing to AIORG

## Process (PR-flow discipline)

- Work happens on feature branches; **no direct pushes to the default branch**.
- Open a **draft PR** -> get tests green -> the owner merges. Contributors do not merge.
- Each PR adds a CHANGELOG entry under `## [Unreleased]` and bumps the version
  (patch = fix, minor = feature) per `docs/VERSIONING.md`: `VERSION` is the single
  source of truth; `node scripts/bump-version.mjs X.Y.Z` syncs `engine/Cargo.toml`.
- Merge commits reference the PR number; releases are tagged `vX.Y.Z` after merge
  (the tag MUST equal `VERSION` — the release workflow guards this).

## Verification gates

Per `AGENTS.md`, done means observed-green tool output:

```powershell
cd engine
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Never mark work done with TODO/FIXME/stub/placeholder left behind.

## Docs

Docs live in `docs/` and must stay truthful to the code — fix the doc in the
same change set when a code change breaks a doc statement.
