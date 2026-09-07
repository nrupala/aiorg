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

- Acceptance integration live run d7252703351f40a7b1d7928600d3bb1c — bwrap command printf accepted PASS; role artifacts, SQLite ledger, and keyed certificate emitted.

- Corrective-edit integration unit suite — 10 tests PASS: strict patch parsing, protected-path rejection, backup/apply/rollback, ΔV, verifier, WAL. Live model-driven patch E2E was attempted but blocked by router model-load failure for Qwen3.5-0.8B-Q8_0; no product files were changed.

- Live local corrective-edit E2E — PASS using llama.cpp Qwen3.5-9B-Q8_0 directly at 127.0.0.1:52544 (no Ollama): aiorg correct created fixed.txt through strict model patch JSON, scoped executor, backup protocol, and bwrap acceptance. Acceptance output: passed; summary: Created fixed.txt with content 'fixed' to satisfy acceptance criteria.

## Certified sovereign-core delivery (2026-09-06)

- AIORG run: 9a920eee497d4c2a9462661e306927e7`n- Model: local llama.cpp Qwen2.5-Coder-7b-instruct-q8_0; no Ollama; no cloud dependency.
- Project: D:\research\sovereign-core`n- Acceptance: sovereign-core browser/unit suite under WSL/bwrap — **69/69 passed**.
- Ledger: verified ledger_chain=true.
- Certificate hash: cdcfcc40a877c059f5a218f630a9526bae00771202f58b798c6edf77718f3b87.
- Certificate producer/verifier: iorg-release / iorg-guard.
- Duration: 104.4 seconds; exit 0.

