# AIORG

AIORG is a local-first, evidence-gated software delivery engine. It models a
software company as nine role procedures and enforces their handoffs through
artifacts, deterministic gates, a SQLite WAL ledger, scoped execution, WSL
bubblewrap acceptance tests, convergence checks, and independent certificates.

AIORG is designed to run with OpenCode or another agent host. The host supplies
interactive intent; AIORG supplies repeatable delivery policy and evidence.

## Current status

**Operational foundation; certified artifact delivery; project-editing loop
available through constrained corrective patches.**

The current release has verified:

- 11 Rust unit/integration tests passing.
- `cargo clippy --all-targets -- -D warnings` passing.
- SQLite WAL event and artifact ledger with tamper detection.
- Scoped executor rejecting absolute paths, traversal, `.git`, and `.aiorg`.
- Backup/apply/rollback corrective patch protocol.
- WSL/bwrap process and network-isolated acceptance execution.
- ΔV non-widening and stall/maximum-cycle guards.
- Independent producer/verifier certificate checks.
- Local, hybrid, and cloud per-role routing configuration.
- stdio MCP server with `aiorg_run`, `aiorg_status`,
  `aiorg_certificate_verify`, and `aiorg_ask_role` tools.
- A real sovereign-core delivery: run
  `9a920eee497d4c2a9462661e306927e7`, acceptance **69/69**, ledger chain
  verified, certificate issued.

The remaining engineering boundary is explicit: AIORG does not claim that every
role output is correct merely because the model generated it. Acceptance tests,
ledger evidence, and independent verification are required for release.

## Architecture

```text
OpenCode / MCP client
        |
        v
AIORG CLI or stdio MCP server
        |
        +-- role router: local llama.cpp, hybrid fallback, or cloud
        +-- SQLite WAL ledger and artifact hashes
        +-- scoped patch executor with backup/rollback
        +-- WSL/bwrap acceptance runner
        +-- ΔV convergence guard
        +-- independent certificate verifier
        |
        v
Project repository + test evidence + certificate
```

The default local endpoint is the llama.cpp OpenAI-compatible router at
`http://127.0.0.1:8830`. Ollama and Docker are not runtime dependencies.

## Repository layout

- `engine/src/main.rs` — CLI and role pipeline.
- `engine/src/config.rs` — provider and per-role routing configuration.
- `engine/src/store.rs` — SQLite WAL ledger and hash-chain verification.
- `engine/src/executor.rs` — scopes, patch validation, backups, rollback, bwrap.
- `engine/src/converge.rs` — ΔV and stall controls.
- `engine/src/verifier.rs` — independent certificate signing/verification.
- `engine/src/mcp.rs` — stdio MCP JSON-RPC server.
- `engine/config/aiorg.toml` — committed provider/model policy.
- `engine/config/providers.local.toml` — local secrets; gitignored.
- `engine/config/mcp.json` — MCP registry policy.
- `org/roles/` and `org/sop/` — role and procedure contracts.
- `docs/` — architecture, SOPs, deployment, release, and OpenCode guidance.
- `TEST_RESULTS.md` — observed verification evidence.

## Prerequisites

- Windows 11 with Rust stable and Cargo.
- WSL2 with `bwrap` available inside the selected distribution.
- Node.js 20+ for JavaScript acceptance gates and MCP helpers.
- A running llama.cpp router at `127.0.0.1:8830` for local inference.
- Optional free cloud provider keys in the ignored
  `engine/config/providers.local.toml`.

No Ollama or Docker installation is required or used.

## Build and verify

From `D:\aiorg\engine`:

```powershell
cargo fmt --all
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build
..target\debug\aiorg.exe doctor
..target\debug\aiorg.exe sandbox wsl
```

The executable name in the code block is `target\debug\aiorg.exe`; the control
character above is not part of the command. Use this exact command if copying:

```powershell
.\target\debug\aiorg.exe doctor
```

## Run a delivery

```powershell
.\target\debug\aiorg.exe run `
  "Deliver the requested project outcome" `
  --project D:\path\to\project `
  --acceptance "cargo test --all-targets"
```

For a focused corrective cycle:

```powershell
.\target\debug\aiorg.exe correct `
  "Fix the failing acceptance test" `
  --project D:\path\to\project `
  --acceptance "cargo test --all-targets"
```

A corrective model response must be strict patch JSON. AIORG validates paths,
backs up overwritten files, applies only scoped edits, runs bwrap acceptance,
and rolls back on failure.

## MCP integration

OpenCode can launch the local MCP server with:

```json
{
  "aiorg": {
    "type": "local",
    "command": ["D:\\aiorg\\engine\\target\\debug\\aiorg.exe", "mcp"],
    "enabled": true,
    "timeout": 30000
  }
}
```

Restart OpenCode after changing its configuration. Inspect discovery with:

```powershell
opencode mcp list
```

## Evidence and release policy

A release is not complete from model output alone. The release gate requires:

1. Clean build, formatting, tests, and clippy.
2. Real acceptance execution in WSL/bwrap.
3. Artifact hashes and SQLite WAL chain verification.
4. Certificate with distinct producer and verifier identities.
5. Documentation matching the observed commands and limitations.
6. A reviewable Git commit and pull request.

See `docs/opencode-aiorg-guidance.md` and `docs/RELEASE-CHECKLIST.md`.

## License

Apache-2.0. See `LICENSE` and `NOTICE`.
