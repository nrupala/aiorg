# 07 — Execution Plan and Delivery Status

Status: M1-M7 runtime foundation and artifact pipeline implemented; certification evidence is recorded in `TEST_RESULTS.md`. Remaining work is tracked explicitly below and is not silently marked complete.
Methodology: Apache-style milestones, each with a **verifiable exit condition**
proven by real tool output; docs updated in the same change set as code.

---

## Milestones

### M1 — Scaffold, config, store
Workspace + crates per `04-architecture.md` §2; `aiorg.toml` loader; SQLite
store with idempotent migrations (runs/tasks/artifacts/events/certificates/
learnings) and hash-chained event append.
**Exit:** `cargo test --all-targets` green; `clippy -D warnings` clean;
migrations re-run safe; `aiorg doctor` binary skeleton reports env truthfully.

### M2 — Provider + role engine
OpenAI-compatible client (router/ollama), SSE accumulation,
`reasoning_content` merge, timeout/retry/circuit-breaker, JSON-Schema
validate-retry envelope; role profiles loaded from `org/roles/*.toml`.
**Exit:** integration tests vs stub server (malformed-output fixture → retry →
escalate); live smoke: `aiorg ask release --dry-run` returns schema-valid
artifact against :8830.

### M3 — Gates + sandbox
G1–G3 deterministic checkers; LX language-gate runners (JS/TS, Python, Rust,
C, SQL, HTML/CSS/XML/JSON); sandbox crate (WSL bwrap scope default, Job-Object
host fallback) with kill-tree + OOM detection.
**Exit:** every language gate passes on its own passing fixture and fails on
its failing fixture; sandbox kills an over-MemoryMax child (observed oom-kill);
host mode blocks blocklisted commands.

### M4 — CONVERGE pipeline (S1–S4)
Dispatcher state machine compiling `org/sop/*.toml`; role sequencing with
RBAC file-scopes; ΔV≤0 enforcement, recursion/stall guards, budgets;
escalation bundles.
**Exit:** golden-path E2E (one-line JS brief → all tasks converge → evidence
complete); forced-failure E2E proves gap never increases and escalation exit=2.

### M5 — Security, release, certificate (S5–S7)
R7 scans; R8 assembly; G6 ledger-consistency check + HMAC certificate;
retro learnings write-back tuning future budgets.
**Exit:** graded fixture suite T1–T5 style across ≥5 languages (JS, TS, Python,
Rust, C, plus SQL migration fixture and HTML/CSS/XML lint fixtures) all certify;
tampered artifact rejected at G6.

### M6 — Surfaces: CLI completeness, SSE serve, MCP server
All §7 CLI commands real; `serve` SSE status on :8850; MCP tools
(`aiorg_ask_role`, `aiorg_run`, `aiorg_status`, `aiorg_certificate_verify`)
over stdio (+HTTP :8851 opt).
**Exit:** opencodelocal lists and successfully calls `aiorg_ask_role` remotely;
replay reproduces a run's certificate hash identically; port-collision fail-fast
proven.

### M7 — Hardening, autonomy levels, deployer validation
L0→L2 config machinery with graduation rule; soak run unattended within
budgets; fresh-environment walkthrough of `06-deployer-guide.md` corrected to
match reality; registries updated (verified entries only).
**Exit:** one L1 soak (multi-task project) completes with zero policy
violations; deployer guide executed verbatim end-to-end by a second party
(the owner or a fresh session) without undocumented steps.

## Test matrix (minimum)

| Area | Cases |
|---|---|
| Store | migrations ×2, chain verify, tamper detect, crash-reopen WAL |
| Provider | stub ok/timeout/500/malformed-schema/breaker-open |
| Gates | per-language pass+fail pairs; checklist parsers negative tests |
| Sandbox | normal run, MemoryMax kill, kill-tree grandchildren, host-mode blocklist |
| Converge | 1-cycle fix, multi-cycle, widening-reject rollback, stall escalate |
| E2E | golden JS brief · Rust CLI brief · Python tool brief · C util brief · SQL migration brief |
| Surfaces | CLI exit codes; SSE frame shape; MCP list/call via real client |

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| 8B-class model quality ceiling on hard tasks | Medium | cycles↑ / escalations↑ | route-up ladder (14B, DS-V4-Flash), planner scoping smaller tasks, honest escalation |
| Router model-swap latency thrash across stages | Medium | minutes lost per run | stage ordering by model adjacency; keep Qwen3-8B resident for 6 of 9 roles |
| WSL unavailable/degraded | Low | sandbox falls back | host Job-Object mode is a full gate-executor; wsl is optimization, not dependency |
| Schema discipline too strict for small models | Medium | retries exhaust | schema-retry loop tuned in M2 fixtures; profile-level prompt hardening |
| Doc drift during build | High | trust erosion | repo law: doc fix required in same commit as behavior change |

## Out of scope (v1), explicitly

Red Team role · DevOps/SRE agent · GUI dashboard (SSE text UI only) · cloud
anything · Docker anything · self-modifying org config.

## Approval gate

Owner approves this document → M1 begins in the next session with registry
consult first (CR reuse where applicable), decision-log entry at each
milestone boundary, and no code outside `D:\aiorg`.
