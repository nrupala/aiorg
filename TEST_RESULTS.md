# AIORG Test Results

## 2026-09-06

- `cargo fmt --all` — PASS
- `cargo test --all-targets` — PASS, 2 passed
- `cargo clippy --all-targets -- -D warnings` — PASS
- `cargo run --quiet -- doctor` — PASS, router healthy; WSL available
- `cargo run --quiet -- gate rust` — PASS
- Full local role pipeline — PASS, run `b96fdb360e474703b79cf4b60eabeb4a`
  - Nine role artifacts generated through the llama.cpp router
  - Distinct SHA-256 artifact hashes observed
  - Ledger written with producer/verifier fields
  - Certificate written with deterministic ledger hash
  - Status: `artifact_pipeline_complete`

## Scope limitation

This run proves the provider-backed nine-role artifact pipeline. It does not yet
prove the M4 project-editing loop: scoped write tools, sandboxed acceptance-test
execution, corrective cycles, delta-V enforcement, or full release verification.
Those remain implementation work and are not marked complete.
