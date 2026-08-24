# 02 — Standard Operating Procedures (SOP)

Status: DRAFT for owner approval. This document is the pipeline law: stages,
procedures, gates, message protocol. The runtime executes this document
literally (SOP files under `org/sop/` compile into the state machine).

---

## 1. Pipeline overview

```
S0 INTAKE → S1 REQUIREMENTS → G1 → S2 ARCHITECTURE → G2 → S3 PLANNING → G3
                                                                    │
     ┌──────────────────────────────────────────────────────────────┘
     │  for each task in DAG order:
     ▼
 S4 IMPLEMENT  ─ R4 Engineer(edit+build) ──► R5 Reviewer(G4a)
                     ▲                            │ approve
                     │                            ▼
                     │                   R6 QA sandbox run (G4b)
                     │                            │ fail
                     └──── corrective diff ◄──────┘   ΔV ≤ 0 enforced
     │ all tasks converged
     ▼
 G4 ──► S5 SECURITY (R7, G5) ──► S6 RELEASE (R8, G6: certificate)
                                              │
                                              ▼
                                    S7 RETROSPECTIVE (learnings → store)
```

**Fast path:** the Dispatcher may collapse S1–S3 into a single *micro-spec*
(one artifact satisfying G1–G3 checks together) when estimated size ≤ 150 LOC,
no new dependencies, and risk = low. The CONVERGE loop of S4 is never skipped.

## 2. Stage procedures

### S0 INTAKE — Dispatcher
1. Health check engines (`GET :8830/health`, `GET :11434/api/tags`); fail fast
   with remediation hints if down.
2. Classify brief: new-project | modify-existing | role-only request.
3. Allocate `run_id` (ULID), create `<project>/.aiorg/runs/<run_id>/`.
4. Emit ledger event `run.started`.

Role-only requests (`aiorg ask <role>`) skip to that role with a synthetic
single-stage plan and still produce evidence + ledger events.

### S1 REQUIREMENTS — R1
Procedure: read brief; if existing repo, scan README/tree (read-only); draft
PRD YAML+MD; self-check against completeness checklist; emit PRD.
**Gate G1 (deterministic):**
- [ ] every user story has ≥1 given/when/then criterion
- [ ] no empty required sections
- [ ] open_questions list present (may be empty only with Principal waiver)
- [ ] out_of_scope non-empty (forces explicit boundary)
Automated as checklist id `G1-completeness` (code parses front matter).

### S2 ARCHITECTURE — R2
Procedure: derive components from stories; define interfaces; choose deps by
rule "reuse local registry entry > stdlib > well-known crate"; write ADRs for
every choice a future reader would question.
**Gate G2:** every story id maps to ≥1 interface in `interfaces.json`;
declared external services = 0 unless PRD NFR demands one; dep list passes
local-first policy (no cloud SDKs without Principal waiver).

### S3 PLANNING — R3
Procedure: split design into tasks ≤ ~300 est-LOC each with file scopes;
author acceptance tests per task (paths under `.aiorg/tests/<task-id>/`);
compute critical path.
**Gate G3:** DAG acyclic (topo-sort in code); each task has ≥1 acceptance test;
scopes are disjoint-or-explicitly-shared; sum(est_loc) within budget band
(from policy) else auto re-plan once then escalate.

### S4 IMPLEMENT (per task) — R4→R5→R6 CONVERGE loop
This is the mathematical core. Per cycle k with gap g_k (fraction of failing
acceptance checks):
1. **transform:** Engineer produces diff constrained to scope.
2. **assess:** Reviewer static gate (G4a), then QA runs acceptance tests in
   sandbox (G4b). Assessors never see transform rationales.
3. **correct:** failure text (compiler/test output, review findings) is the
   corrective input for next cycle.
Loop halts when g=0. Invariants enforced **in code**: gap never increases
(ΔV≤0; widening proposals rejected + rolled back); max_cycles (default 8),
recursion guard max_r (default 3), stall detection (ρ ≥ 1.0 over window).
Breach ⇒ escalate with full evidence bundle.

### S5 SECURITY — R7
Procedure: secret scan (regex + entropy heuristics over diff), injection
surface walk (any new exec/SQL/shell/template path), least-privilege review.
**Gate G5:** verdict clear; secrets clean; high findings = block.

### S6 RELEASE — R8
Procedure: version per semver; changelog from task diffs; tag proposal; notes.
**Gate G6:** runtime verifies G1–G5 all green **in the ledger**, evidence
hashes recompute, producer≠verifier on every artifact, then issues the signed
Convergence Certificate (HMAC-SHA256, key from env `AIORG_RUN_KEY`; never in
context). Autonomy < L2 ⇒ certificate status `pending_principal`.

### S7 RETROSPECTIVE — machinery (+ optional LLM summary)
Append learnings `{task_class, cycles_used, stall?, model_route, notes}` to
store; update project memory; close run event. Learnings feed future budget
defaults (self-improvement loop).

## 3. Message & artifact protocol

Agents never exchange free-form chat. Two channels only:

1. **Artifacts of record** — files in `<project>/.aiorg/runs/<run_id>/`
   (`prd.md`, `design.md`, `interfaces.json`, `tasks.json`, diffs,
   `*-report.json`, `certificate.json`). These are git-committable truth.
2. **Typed envelopes** (in-process / ledger):
```json
{
  "v": 1,
  "from": "R6_qa",
  "to": "O2_dispatcher",
  "type": "gate.verdict",
  "run_id": "01J...",
  "task": "T-003",
  "artifact_refs": ["qa-report.json"],
  "evidence": {"cmd": ".aiorg/tests/T-003/run.sh", "exit": 1, "sha256": "..."},
  "ts": "2026-08-23T09:15:00Z"
}
```

Rules: every claim field must reference an artifact or evidence hash; envelopes
are appended to the ledger verbatim; roles cannot address other roles directly
(all routing through O2) — this kills hidden coordination paths.

## 4. Escalation path

Trigger conditions: any gate fail after budget; schema-retry exhausted (2);
stall/recursion breach; engine unhealthy after 1 retry; ambiguity flagged by
R1/R2 marked `blocking`. Escalation = pause run, assemble evidence bundle
(relevant artifacts + last N ledger events), surface via CLI/SSE/MCP with a
concrete question list. Principal answers resume the run (`aiorg resume`).

## 5. Gate catalog

| ID | Type | Executor | Pass condition |
|---|---|---|---|
| G1 | checklist-as-code | parser | completeness rules §S1 |
| G2 | consistency | matcher | story↔interface coverage |
| G3 | structural | topo/checker | acyclic DAG, tests present, budget band |
| G4a | LLM review | R5 instance | approve, no high severity |
| G4b | tests in sandbox | QA runner | exit 0 all suites |
| G5 | security | scans + R7 | clear/clean |
| G6 | certificate issuer | runtime crypto | ledger consistency + signatures |
| LX | language gates | toolchain | see guardrails doc table |

Language gates (LX) run inside G4b before acceptance suites: JS/TS
(`node --check`, `tsc --noEmit`, eslint when configured), Python (`pytest`,
ruff/mypy when configured), Rust (`cargo test --all-targets`, clippy `-D
warnings`), C (`cmake --build`, gcc/clang `-Wall -Wextra -Werror`), SQL
(sqlite parse + migration dry-run on temp DB), HTML/CSS/XML/JSON
(prettier --check / tidy where installed; strict JSON/YAML/TOML parse always).
Missing toolchain for a language ⇒ that language's tasks cannot be planned
(doctor reports this before S1).
