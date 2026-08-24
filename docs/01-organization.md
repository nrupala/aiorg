# 01 — The AIORG Organization

Status: DRAFT for owner approval · Scope: this document defines the company in
full — charter, principles, roster, per-role specifications, autonomy levels.
The SOP pipeline lives in `02-sop.md`; controls in `03-guardrails.md`.

---

## 1. Charter

AIORG is a single-tenant, locally-run software development company whose
employees are role-specialized LLM agents governed by deterministic machinery.
Its purpose is to produce verified, releasable software for its sole Principal
(the owner) and to make every individual role callable on demand against the
owner's ongoing codebases.

Success definition (the only metric that counts): a run ends in a **signed
Convergence Certificate** — independent tests pass in sandbox, review and
security gates are clean, evidence is ledgered, and nothing was self-verified.

## 2. Operating principles (evidence base)

| # | Principle | Why (source) |
|---|---|---|
| P1 | Structure beats persona | EMNLP 2025 *Principled Personas*: expert personas are positive-or-non-significant; irrelevant persona detail drops performance ~30 pts. Wharton 2025: personas don't improve factual accuracy. MetaGPT ICLR 2024: gains come from SOPs + structured artifacts + executable feedback ("Code = SOP(Team)"). |
| P2 | Verifier ≠ producer | MAST (NeurIPS 2025): verification failures are 21% of multi-agent failures; Goodhart guard requires an independent assessment channel. |
| P3 | Contain at handoffs | MAST: errors propagate through handoffs; 95%-reliable steps compound to ~36% over 20 hops → minimize hops, gate every hop deterministically. |
| P4 | Artifacts, not chat | Free-form inter-agent chat is the top carrier of MAST inter-agent misalignment (37%). All communication = files + typed envelopes. |
| P5 | Policy in code | Prompt instructions are not enforcement. Gates, budgets, RBAC are executed by the runtime. |
| P6 | Local-first | All inference via llama.cpp router :8830 / Ollama :11434, loopback only. No cloud in the delivery path. |
| P7 | Honest status | Nothing is "done" without observed tool output; unproven capability is labeled as such. |

## 3. Org chart

```
                    ┌───────────────────────────┐
                    │  PRINCIPAL (owner, human) │   briefs · approvals · arbitrations
                    └─────────────┬─────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │  DISPATCHER  (O2)         │   routing · gates · escalation · audit emit
                    └──┬─────┬─────┬─────┬──────┘
        ┌──────────────┘     │     │     └───────────────┐
        ▼                    ▼     ▼                     ▼
  DELIVERY LINE                                        RELEASE LINE
  R1 Analyst ─► R2 Architect ─► R3 Planner             R8 Release Manager
                        │                                   ▲
                        ▼ per task (CONVERGE loop)          │ green bundle only
              R4 Engineer ─► R5 Reviewer ─► R6 QA ─────────┤
                                    (independent)          │
                              R7 Security Officer ─────────┘
```

v1 roster is **9 roles** (owner-approved core-first scope). System functions
that would otherwise need staff roles are performed by **machinery, not LLMs**
— this is deliberate zero-trust design:

| Former staff role | Performed by |
|---|---|
| Scribe/Auditor | The runtime itself emits append-only ledger events (an LLM scribe could falsify records; code cannot). |
| Librarian/Knowledge curator | Retro stage writes learnings JSON via `store` (deterministic), reviewed next run. |
| DevOps/SRE | Deployer guide + engine health checks (`aiorg doctor`); v2 may add an SRE agent. |
| Red Team | v2 adversarial stage. |

## 4. Role specifications

Conventions:
- **Model ids** are llama.cpp router preset names from
  `router-models.ini` (served at `http://127.0.0.1:8830/v1`). Fallbacks route to
  Ollama pinned variants (`granite4:3b-oc`, `qwen3:1.7b-oc`) only for light work.
- Every role's output must validate against its JSON Schema (stored in
  `org/schemas/`). Invalid output ⇒ schema-retry loop (max 2) ⇒ escalate.
- Tool allowlists are enforced by the runtime RBAC layer, not by politeness.

### O2 — Dispatcher (chief of staff)
- **Mission:** own the pipeline end-to-end: classify the brief, choose fast-path
  vs full SOP, sequence stages, evaluate gate verdicts, escalate to Principal,
  emit audit events.
- **Trigger:** `aiorg run "<brief>"`, `aiorg resume <run-id>`, or MCP call.
- **Inputs:** brief; run state; gate verdicts. Never raw secrets; never producer
  rationales.
- **Output contract:** `run-plan.json` `{run_id, mode: full|fast, stages[],
  budgets{max_cycles,max_r,stall_rho,time_s}, autonomy}`.
- **Tools:** store read/write, gate runner invocation. NO repo write access.
- **Model:** `Qwen3-8B-Q4_K_M` (resident primary; no swap cost).
- **Gate:** plan validates; DAG acyclic; budgets within policy ceilings.
- **Escalate when:** any gate fails after budget; ambiguity that changes scope;
  port/engine unhealthy after remediation.

### R1 — Requirements Analyst
- **Mission:** convert a brief into an unambiguous PRD with testable acceptance
  criteria; surface every open question explicitly instead of guessing.
- **Input:** brief (+ interview answers if Dispatcher relays).
- **Output:** `prd.md` with YAML front matter: `{id, title, users[], stories[]
  (each with given/when/then), nfrs[], out_of_scope[], open_questions[]}`.
- **Tools:** read-only repo scan (if assignment targets existing code).
- **Model:** `Qwen3-8B-Q4_K_M`.
- **Gate G1:** completeness checklist-as-code — every story has ≥1 criterion;
  no empty sections; open_questions either resolved by Principal or explicitly
  deferred with owner sign-off.
- **Anti-goals:** no solutioning (that's R2's job); no silent assumptions.

### R2 — Architect
- **Mission:** turn the PRD into the smallest design that satisfies it: module
  boundaries, public interfaces, data model, dependency choices (prefer
  registry/local-first reuse), ADRs for non-obvious decisions.
- **Input:** approved PRD only.
- **Output:** `design.md`: components table, interface signatures, data model,
  ADR list, risk notes; `interfaces.json` (machine-checkable).
- **Model:** `DeepSeek-R1-0528-Qwen3-8B-Q4_K_M` (reasoning; provider reads
  `reasoning_content`), fallback `Qwen3-14B-Q4_K_M`.
- **Gate G2:** interface-consistency check (every story maps to ≥1 interface);
  no unspecified external service calls; local-first rule respected.
- **Anti-goals:** no code bodies; no gold-plating beyond NFRs.

### R3 — Task Planner
- **Mission:** decompose design into an acyclic task DAG where **acceptance
  tests are authored here, before implementation exists** — the structural
  guarantee that QA verifies independently of the Engineer.
- **Input:** `design.md` + `interfaces.json`.
- **Output:** `tasks.json` `{tasks:[{id,title,files_scope[],depends_on[],
  acceptance_tests:[paths],est_loc,risk}], critical_path}`.
- **Model:** `Qwen3-8B-Q4_K_M`.
- **Gate G3:** DAG acyclicity (code-enforced), every task has ≥1 acceptance
  test path and scoped file list; total est within budget band else re-plan.
- **Anti-goals:** never let one task span the whole repo ("scoped blast radius").

### R4 — Engineer
- **Mission:** implement exactly one task inside its file scope, run its own
  build/self-test, deliver diff + honest evidence. Self-tests passing is
  necessary, never sufficient.
- **Input:** one task + scoped files + interfaces.json. Does NOT see QA test
  bodies (prevents teaching-to-the-test; sees criteria titles only).
- **Output:** unified diff + `impl-evidence.json` {build_cmd, exit, log_tail}.
- **Tools:** scoped read/write (RBAC paths from tasks.json), command exec
  (blocklist + Job Object), git.
- **Model:** `qwen2.5-coder-7b-instruct-q4_k_m`; fallback
  `dspark-DeepSeek-V4-Flash-0731-BF16` for hard tasks, `Qwen2.5-Coder-14B-Instruct-Q3_K_L`.
- **Gate (in-loop):** builds clean; no blocklisted ops; diff confined to scope.
- **Anti-goals:** drive-by refactors; touching acceptance tests; scope creep.

### R5 — Reviewer
- **Mission:** independent static review of the diff against checklist
  (correctness vs interfaces, error handling, naming, complexity, security
  smells, doc truth).
- **Input:** diff + interfaces.json + checklist. Never the Engineer's
  rationale; never the same completion session as production.
- **Output:** `review.json` `{verdict: approve|block, findings:[{severity,
  location, note}], checklist:{item:pass|fail}}`.
- **Model:** `Qwen3-8B-Q4_K_M` (fresh instance; different route than coder model).
- **Gate G4a:** verdict approve AND zero severity≥high findings.
- **Anti-goals:** style nitpicks that belong in lint; rewriting the diff.

### R6 — QA Engineer
- **Mission:** run the pre-authored acceptance tests plus generated edge cases
  **in the sandbox**; produce the run report that is the primary convergence
  signal.
- **Input:** acceptance tests + built artifact. Writes NO product code.
- **Output:** `qa-report.json` {suite results, coverage delta if available,
  sandbox_id, timings}.
- **Tools:** sandbox exec only (bwrap/systemd-run scope or Job Object host mode).
- **Model:** `qwen2.5-coder-7b-instruct-q4_k_m` (edge-case generation).
- **Gate G4b (CONVERGE assess step):** all suites green under sandbox; failure
  text becomes corrective input for R4; loop enforces gap ΔV≤0, recursion
  guard maxR=3, stall detection rho≥1.0 → escalate.
- **Anti-goals:** modifying tests to pass; declaring green without sandbox exit codes.

### R7 — Security Officer
- **Mission:** threat-model the change: secret leakage, injection surfaces
  (prompt/command/SQL/path), dependency risk, least privilege of new code.
- **Input:** diff + design + security checklist (OWASP LLM-top-10-informed).
- **Output:** `security-report.json` {verdict, findings[], scans:{secrets:
  clean|hits}}.
- **Model:** `DeepSeek-R1-0528-Qwen3-8B-Q4_K_M`.
- **Gate G5:** verdict clear; zero secret hits; findings below threshold or
  waived by Principal in writing (ledgered).
- **Anti-goals:** rubber-stamping; blocking on theoreticals without PoC note.

### R8 — Release Manager
- **Mission:** assemble the release: version bump, CHANGELOG entry from merged
  diffs, tag proposal, release notes draft, and request certificate issuance.
- **Input:** all green gate artifacts.
- **Output:** `release.json` {version, changelog_excerpt, tag, notes} +
  certificate request.
- **Model:** `Qwen3-8B-Q4_K_M`.
- **Gate G6:** runtime issues **Convergence Certificate** only when: G1–G5
  green in ledger, evidence hashes match, verifier identities ≠ producer
  identities. At autonomy < L2 the certificate pends Principal approval.

## 5. Autonomy levels

| Level | Gates | Releases | Human load |
|---|---|---|---|
| **L0** (default) | Auto-run, each verdict shown | Principal approves every release | High visibility |
| **L1** | Auto, silent unless fail | Principal approves releases | Low |
| **L2** | Auto | Auto within scope/budget caps declared in the brief | None until escalation |

Graduation rule: a level is earned per-project by 3 consecutive certified runs
without escalation; regressions drop the project back one level. Levels are
config (`aiorg.toml` per project), never prompt states.

## 6. What AIORG deliberately does NOT do (v1)

- No long-lived daemons making unilateral decisions between runs.
- No cloud inference anywhere in the delivery path.
- No self-modification of its own org config by agents.
- No Docker-based anything.
- No claim of capability beyond verified fixtures (honesty boundary).
