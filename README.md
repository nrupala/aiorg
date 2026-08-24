# AIORG — Orchestrated AI Software Development Company

A self-contained, local-first multi-agent software company that runs entirely on
your machine (llama.cpp router + Ollama), executes role-specialized agents through
SOP-gated pipelines with deterministic verification, and exposes **each role as an
individually callable function** for your ongoing code assignments.

**Status: BUILD IN PROGRESS — M1–M7 incremental milestones underway; docs v0.1.0-draft; see `docs/07-execution-plan.md` for exit conditions.**

## The one-paragraph pitch

AIORG encodes a software company as 9 specialized roles (Dispatcher, Requirements
Analyst, Architect, Task Planner, Engineer, Reviewer, QA Engineer, Security
Officer, Release Manager) connected by structured artifact contracts — never
free-form chat — where every handoff passes a deterministic gate (tests, lint,
build, checklist-as-code) and no agent is ever allowed to verify its own work.
It runs on the llama.cpp router (:8830) and Ollama (:11434) already installed on
this machine, keeps a tamper-evident append-only audit ledger, can replay any run
deterministically, and lets you invoke any single role on any existing codebase:

```powershell
aiorg ask reviewer --path D:\myproj          # one role, your live code
aiorg run "build a pomodoro CLI"             # full company pipeline
```

## Documentation map (read in order)

| Doc | Contents |
|---|---|
| `docs/01-organization.md` | Charter, operating principles, all 9 roles in full detail, autonomy levels |
| `docs/02-sop.md` | Standard Operating Procedures: stage-by-stage pipeline S0–S7, gates, message protocol |
| `docs/03-guardrails.md` | Zero-trust / zero-knowledge / transparency controls, MAST failure counter-map |
| `docs/04-architecture.md` | Flow of code: crates, data flow, store schema, provider layer, CLI/MCP reference |
| `docs/05-walkthrough.md` | End-to-end narrative: one real assignment through every stage and gate |
| `docs/06-deployer-guide.md` | Prerequisites, install, engine bring-up, first-run verification, operations, troubleshooting |
| `docs/07-execution-plan.md` | Build milestones M1–M7 with exit criteria, test matrix, risk register |
| `docs/VERSIONING.md` | App versioning standard: VERSION source of truth, tag==VERSION guard, bump script, release workflow |

## Non-negotiable design rules

1. Structure beats persona: gains come from SOPs, contracts, and independent
   verification — never from "the model believes it's an expert".
2. Verifier ≠ producer. Ever.
3. Every claim carries machine-verifiable evidence; releases require a signed
   Convergence Certificate.
4. Agents communicate only through artifacts on disk plus typed envelopes.
5. All policy is enforced in code, not prompt instructions.
6. Local-first: zero cloud calls in the delivery path.
7. No Docker. Sandbox = WSL/bwrap or Windows Job Objects.

D:\Harness is a separate tool (engine management). AIORG shares no code with it.
