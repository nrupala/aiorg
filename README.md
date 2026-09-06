# AIORG — Orchestrated AI Software Development Company

A self-contained, local-first multi-agent software company that runs entirely on
your machine (llama.cpp router :8830), executes role-specialized agents through
SOP-gated pipelines with deterministic verification, and exposes **each role as an
individually callable function** for your ongoing code assignments.

**Status: M1-M3 runtime foundation and the nine-role artifact pipeline are operational; the M4 project-editing CONVERGE loop remains incomplete.** The current binary produces role artifacts, ledger hashes, and certificates through the local llama.cpp router, but does not yet edit target repositories or execute QA acceptance suites automatically.

## The one-paragraph pitch

AIORG encodes a software company as 9 specialized roles (Dispatcher, Requirements
Analyst, Architect, Task Planner, Engineer, Reviewer, QA Engineer, Security
Officer, Release Manager) connected by structured artifact contracts — never
free-form chat — where every handoff passes a deterministic gate (tests, lint,
build, checklist-as-code) and no agent is ever allowed to verify its own work.
It runs on the llama.cpp router (:8830) already installed on
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
| `org/sop/requester_template.toml` | Template for requesters to submit briefs + success criteria + budget; AIORG Dispatcher sequences through all 9 roles via CONVERGE pipeline |

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

## Requester SOP Template

Use the template at `org/sop/requester_template.toml` to request AIORG deliver a
specific outcome deterministically. Fill in Sections 1–3 (Brief, Success Criteria,
Budget Cap) at minimum, save the file, and AIORG's Dispatcher will sequence through
all 9 v1 roles via the CONVERGE pipeline. Outcomes include:

- Convergence certificate (HMAC-SHA256, producer≠verifier)
- ΔV≤0 budget enforcement report
- Per-role gate pass/fail log
- Token usage accounting
- Escalation bundle (if any gate failed)

**Example workflow:**

```powershell
# 1. Copy and fill the template
Copy-Item 'D:\aiorg\org\sop\requester_template.toml' 'D:\aiorg\org\sop\my-project.toml'
# Edit my-project.toml with your brief, success criteria, and token cap

# 2. Run AIORG on your brief
aiorg run "my brief here"

# 3. Check results
aiorg status
aiorg certificate_verify
```
