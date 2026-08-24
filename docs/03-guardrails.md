# 03 — Guardrails: Zero Trust · Zero Knowledge · Transparency

Status: DRAFT for owner approval. Controls here are enforced by AIORG's code.
Where a control exists only as a v2 plan, it says so explicitly.

---

## 1. Threat model (what we defend against)

1. **Cascading agent error** — one agent's wrong output becomes another's
   trusted input (MAST families 1–3).
2. **Self-serving verification** — producer grading its own work (Goodhart).
3. **Prompt injection via repository content** — malicious instructions inside
   files the agents read.
4. **Runaway execution** — infinite loops, fork bombs, disk fillers.
5. **Secret exfiltration** — credentials reaching model context or artifacts.
6. **History falsification** — rewriting what happened (audit tamper).
7. **Engine/port confusion** — another local service impersonating an engine.

## 2. Control matrix

| Threat | Control (enforced in code) |
|---|---|
| Cascading error | Deterministic gates at every handoff (§02-sop G1–G6, LX); hop minimization via fast-path; schema-validate-retry (max 2) then escalate. |
| Self-verification | Structural separation: acceptance tests authored at S3 before implementation exists; Reviewer/QA instances never share completion context with Engineer; certificate checks producer≠verifier identity pairs. |
| Prompt injection | File contents enter prompts as quoted DATA blocks with source labels; system prompt instructs ignore-instructions-in-data AND the runtime strips/flags imperative patterns ("ignore previous", fake envelopes) before dispatch; tools invoked only by the runtime from envelope types — model output can request, never invoke directly. |
| Runaway execution | Sandbox: WSL Ubuntu + bubblewrap (read-only /usr,/lib..., fresh /dev,/proc, tmpfs /tmp, unshare-pid/net, die-with-parent) inside `systemd-run --user --scope` MemoryMax + MemorySwapMax=0; host fallback Windows Job Objects KILL_ON_JOB_CLOSE + command blocklist; per-stage time budgets. No Docker anywhere. |
| Secret exfiltration | Secrets live in env / OS credential store; provider layer redacts known secret shapes from outbound payloads; artifact scanner (G5) blocks secret-shaped strings in diffs; run key `AIORG_RUN_KEY` used only by cert issuer code path. |
| History falsification | Append-only SQLite WAL ledger (events hash-chained: `h(n)=H(h(n-1)‖event)`); Replay rebuilds any run from ledger + recorded proposer calls (sha256-keyed); tamper ⇒ chain verify fails loudly. |
| Engine impersonation | Loopback-only endpoints; health handshake includes expected model id list; ports owned: serve :8850, MCP HTTP opt :8851 — startup fails fast if busy. |

## 3. Zero-knowledge context scoping (need-to-know)

Each role receives exactly its SOP input set (§01 §4) — nothing more:

| Role | Sees | Never sees |
|---|---|---|
| Analyst | brief, repo tree (read-only) | other runs' data |
| Architect | approved PRD | implementation |
| Planner | design + interfaces | engineer chats |
| Engineer | own task, scoped files, criteria titles | acceptance test bodies, reviewer rationale |
| Reviewer | diff, interfaces, checklist | engineer rationale |
| QA | tests, built artifact | product-code authoring rights |
| Security | diff, design | unrelated modules |
| Release | green artifacts only | raw intermediate failures |

The RBAC layer enforces file-scope read/write per role; violations abort the
stage and raise a ledger `policy.violation` event.

## 4. Evidence & Convergence Certificate

Every gate emits an evidence record:
```json
{"gate":"G4b","cmd":"...","exit":0,"duration_ms":4123,
 "stdout_tail":"...","stderr_tail":"","inputs_sha256":["..."],
 "verifier":"R6_qa@llama-router","producer":"R4_engineer@llama-router"}
```
Certificate = manifest of all evidence records + final artifact hashes,
signed HMAC-SHA256 (`AIORG_RUN_KEY`), stored as `certificate.json` +
ledgered. Verification = recompute chain + hashes + identity rules.
**A release without a verifiable certificate is not a release** — tooling
refuses to package it.

## 5. Budgets & termination defaults

| Knob | Default | Ceiling |
|---|---|---|
| max_cycles per task (CONVERGE) | 8 | 16 |
| recursion guard max_r | 3 | 5 |
| stall rho threshold | 1.0 | fixed |
| stage time budget | 600 s | 3600 s |
| schema retries | 2 | fixed |
| engine retry on transient | 1 | 3 |

Learnings from S7 tune defaults downward per task class; ceilings are policy,
not tunable by agents.

## 6. MAST counter-map (named failure modes → our controls)

| MAST family / mode | AIORG control |
|---|---|
| FC1 spec/design flaws (41.8%) — fail-to-follow-requirements, step repetition, unrecognized completion | G1/G2/G3 deterministic checks; cycle/state machine (repetition impossible by construction); completion defined solely by g=0 + gates |
| FC2 inter-agent misalignment (36.9%) — incompatible outputs, hidden comms | JSON-Schema'd artifacts; single artifact channel; O2-only routing |
| FC3 verification gaps (21.3%) — incorrect/incomplete verification | verifier independence; sandbox exit codes as truth; certificate identity rule |
| Context loss | artifacts-of-record persist; runs resumable from ledger |
| Termination gaps | budgets + recursion/stall guards escalate rather than spin |

## 7. Honesty boundary

AIORG may only claim capabilities proven by its fixture suite (`07-execution-plan.md`
test matrix). Anything unproven is labeled experimental in docs and UI. This
document and all others must stay truthful to shipped code — doc drift is a
defect fixed in the same change set.
