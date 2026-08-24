# 05 — Walkthrough: One Assignment Through the Whole Company

Status: DRAFT for owner approval. A realistic end-to-end narrative showing
exactly what you will see and do, including a failure→correction cycle and an
escalation. Artifact snippets are illustrative of the real schemas.

---

**The brief (you type):**

```powershell
aiorg run "Build a note-taking CLI in Rust called notesr: add/list/search notes,
           stored as markdown files under ./notes" --project D:\sandbox\notesr
```

## S0 — Intake (milliseconds)

```
[aiorg] engine check  llama.cpp :8830 … ok (Qwen3-8B-Q4_K_M resident)
[aiorg] port check    serve :8850 … free
[aiorg] run allocated 01J8XQ… · project D:\sandbox\notesr · autonomy L0 · mode full
```
Ledger: `run.started`.

## S1 — Requirements Analyst (R1)

Reads the brief; asks nothing (no blocking ambiguity) but records one deferred
question ("sync across machines?") into `open_questions` with status deferred.

`prd.md` front matter (abridged):
```yaml
id: PRD-notesr-001
stories:
  - id: S1
    given: "./notes contains *.md"
    when: "notesr list"
    then: "prints titles sorted by mtime"
  - id: S2 {search by substring → matching paths}
  - id: S3 {add via editor or --text}
nfrs: ["cold start < 100ms", "no network"]
out_of_scope: [sync, encryption]
open_questions: [{q: cross-machine sync?, status: deferred-by-principal}]
```

**G1 verdict:** pass (every story has criteria; boundary explicit).

## S2 — Architect (R2)

Reasoning model (`DeepSeek-R1-0528-Qwen3-8B`, thoughts arrive in
`reasoning_content`). Chooses stdlib-only + `serde_json` for an index cache,
ADR-001 explains why not SQLite (NFR: no heavy deps; file count small).
Outputs component table + `interfaces.json`:

```json
{"cli": {"add": "--text STR | opens $EDITOR", "list": "", "search": "QUERY"},
 "storage": {"layout": "notes/YYYYMMDD-slug.md", "index": ".notes-index.json"}}
```

**G2 verdict:** pass (S1–S3 all map to cli/storage interfaces; zero external services).

## S3 — Task Planner (R3)

Emits DAG + **authors acceptance tests before any code exists**:

```
T-001 storage module      scope: src/storage.rs            tests: .aiorg/tests/T-001/
T-002 cli commands        scope: src/main.rs               tests: .aiorg/tests/T-002/  deps T-001
T-003 search + index      scope: src/{search.rs,index.rs}  tests: .aiorg/tests/T-003/  deps T-002
```

**G3 verdict:** pass (acyclic; every task has tests; 3 tasks ≈ 420 est-LOC within band).

## S4 — Implement loop

### T-001 cycle 1
- Engineer (`qwen2.5-coder-7b`) writes `src/storage.rs`; `cargo build` exit 0.
- Reviewer (fresh Qwen3-8B instance) → `review.json`: approve, 1 medium finding
  ("unwrap on malformed filename").
- QA runs `.aiorg/tests/T-001/` in WSL bwrap sandbox: **1 of 3 fails**
  (slug collision). g = 0.33.
- Corrective input to engineer = exact test output. Cycle 2 fixes collision;
  reviewer approves; sandbox green. g = 0 ✔ (ΔV ≤ 0 held: 0.33 → 0).

### T-002 cycles 1–2 — same pattern, converges.
### T-003 cycle 1 → stall guard trip
Engineer's index approach loops (ρ ≥ 1.0 over window, gap flat at 0.5).
Runtime halts, rolls back the widening attempt (gap must never increase), and
**escalates to you**:

```
[aiorg] ESCALATION run 01J8XQ… task T-003 reason stall(rho=1.0, cycles=4)
        evidence: .aiorg/runs/01J8XQ…/escalation.md   (diffs, logs, ledger tail)
        question: allow larger rewrite of search.rs (scope extension), or drop index?
You: aiorg resume 01J8XQ… --answer "drop the json index; plain scan is fine"
```

Planner re-scopes T-003 (ADR-002 appended); engineer converges in 2 cycles.

## S5 — Security Officer (R7)

Secrets scan clean; flags one path-handling finding (filename traversal in
search arg) → fixed same-stage by targeted diff (verifier re-runs LX+tests).

## S6 — Release (R8) & Certificate (G6)

```
version 0.1.0 · CHANGELOG from task diffs · tag v0.1.0 proposed
[aiorg] G6: G1–G5 green in ledger ✓ evidence hashes recomputed ✓ producer≠verifier ✓
[aiorg] Convergence Certificate signed (HMAC-SHA256) — status PENDING_PRINCIPAL (L0)
Approve release? [y/N]: y
[aiorg] RELEASED v0.1.0 · certificate issued · run closed in 14m 32s
```

## S7 — Retrospective (automatic)

Learnings appended: `{task_class: cli-tool, avg_cycles: 1.7, stall_events: 1,
note: json-index approaches stall on this model class}` — next similar brief
starts with planner steered away from that pattern (self-improvement loop).

Ledger now holds 61 hash-chained events; `aiorg audit 01J8XQ…` verifies chain;
`aiorg replay 01J8XQ…` reproduces the run deterministically.

---

## Calling single roles on your existing code (the everyday mode)

```powershell
aiorg ask reviewer --path D:\myproj --in HEAD~2..HEAD     # independent review of last 2 commits
aiorg ask security --path D:\myproj\api                   # threat-model a subsystem
aiorg ask qa       --path D:\myproj --spec .\criteria.md  # independent acceptance suite
aiorg ask architect --path D:\myproj                      # design review of current structure
```

Each returns its schema-valid artifact + evidence + ledger entry, exactly as it
would inside a full run.
