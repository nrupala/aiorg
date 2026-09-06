# OpenCode / AIORG Operating Guidance

This repository is the source of truth for AIORG's software-delivery method.
OpenCode may implement changes, but must preserve the following order:

1. Intake: record the brief, project path, constraints, and success criteria.
2. Requirements: produce testable acceptance criteria; do not silently resolve
   scope-changing ambiguity.
3. Architecture: define interfaces, data boundaries, dependencies, and risks.
4. Planning: split work into the smallest independently verifiable tasks with
   disjoint file scopes where possible.
5. Implementation: make only the scoped change; preserve the artifact ledger.
6. Review: inspect the diff independently from the implementation reasoning.
7. QA: execute the acceptance tests in WSL/bwrap; do not declare success from
   inspection alone.
8. Security: scan secrets, path/command/SQL/template boundaries, and dependency
   changes.
9. Release: verify hashes, ledger chain, certificate producer/verifier identity,
   documentation, and reproducible instructions.

## Deterministic-first rule

The program performs path resolution, file discovery, hashing, budgets, retries,
process isolation, test execution, git state, certificate verification, and
ledger updates. The model supplies judgment and drafts; it must not be asked to
perform deterministic work the runtime can perform.

## Convergence rule

Every corrective cycle records an observed gap. A new gap may not exceed the
previous gap. Equal gaps beyond the stall threshold escalate automatically.
Tests, reviewer findings, and sandbox exit codes are evidence; model claims are
not evidence.

## Change protocol

- Work on a feature branch.
- Back up existing files before mutation.
- Make one coherent milestone per commit.
- Run `cargo fmt --all`, `cargo test --all-targets`, and
  `cargo clippy --all-targets -- -D warnings` before opening a PR.
- Open a PR for review; do not merge it automatically.
- Never target Ollama or Docker.
- Do not mark AIORG complete while any runtime path is a stub, placeholder, or
  unverified claim.
