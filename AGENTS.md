# AGENTS.md — rules for any agent working in this repository

This repo is AIORG: an orchestrated multi-agent software development company.
Until the owner approves `docs/07-execution-plan.md`, this repo is DOCS-ONLY:
do not create implementation code.

## Verification is the definition of done

When implementation begins, every milestone exit condition must be proven by
real tool output:

- Rust: `cargo test --all-targets` AND `cargo clippy --all-targets -- -D warnings`
  both clean. A pass is observed, not assumed.
- TypeScript/JavaScript: `tsc --noEmit`, lint, and tests green when those exist.
- Python: `pytest`; add `ruff`/`mypy` checks if configured.
- C: `cmake --build` clean; gcc/clang `-Wall -Wextra -Werror`.
- Never mark done with TODO/FIXME/stub/placeholder left behind.

## Architecture law (do not violate)

1. Self-contained: no path/git dependencies on other local projects
   (D:\Harness, opencodelocal, oclrust are DIFFERENT tools).
2. Engines: llama.cpp router at http://127.0.0.1:8830/v1 (model ids come from
   `C:\Users\nrupa\.config\opencode\local-engines\router-models.ini`) and
   Ollama at http://127.0.0.1:11434. Loopback only.
3. Reasoning models return text in `message.reasoning_content` — always read
   BOTH `content` and `reasoning_content`.
4. Ports owned by AIORG: serve :8850, optional MCP HTTP :8851. Fail fast if busy.
5. No Docker, ever. Untrusted/test execution goes through WSL Ubuntu +
   bubblewrap + systemd-run scope, or Windows Job Objects on the host side.
6. Artifacts of record live in the assignment project under `.aiorg/`;
   AIORG's own central index DB lives under `D:\aiorg\data\`.
7. Append-only audit ledger; deterministic Replay; verifier != producer.
8. Secrets never enter model context; read from env/config at execution time.

## Process law

- Consult `C:\Users\nrupa\.config\opencode\registries\` indexes before writing
  new code; register anything reusable that gets built (status honest:
  experimental until proven).
- Append session decisions to
  `C:\Users\nrupa\.config\opencode\conversation_decisions.md`.
- Local-first: use on-machine tooling; no cloud calls in the delivery path.
- Docs live in `docs/`; keep them truthful to the code. If code changes break
  a doc statement, fix the doc in the same change set.
