# AIORG Release Checklist

This checklist is mandatory before declaring an AIORG release, PR, or certified
project delivery complete.

## Source and repository

- [ ] Worktree contains only intended changes.
- [ ] Secrets are absent from tracked files and history.
- [ ] Changes are on a feature branch.
- [ ] Commit message is specific and reviewable.
- [ ] Pull request contains scope, risks, and observed verification.
- [ ] No Docker or Ollama runtime dependency was introduced.

## Build and quality

- [ ] `cargo fmt --all` passes.
- [ ] `cargo test --all-targets` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] Provider and MCP configuration parses successfully.
- [ ] No known TODO, FIXME, placeholder, or knowingly unimplemented path is
      included in the claimed release scope.

## Runtime evidence

- [ ] `aiorg doctor` reports the intended provider state.
- [ ] WSL/bwrap sandbox executes a real command.
- [ ] The actual project acceptance suite runs inside the sandbox.
- [ ] Corrective edits are scoped, backed up, and rollback-tested.
- [ ] SQLite WAL chain verifies after the run.
- [ ] Certificate verification succeeds independently of artifact production.
- [ ] Producer and verifier identities differ.

## Documentation

- [ ] Root README states purpose, status, prerequisites, install/run commands,
      architecture, configuration, security boundaries, tests, and license.
- [ ] Documentation commands were executed exactly as written.
- [ ] Limitations and unverified claims are explicit.
- [ ] `TEST_RESULTS.md` records the command and observed result.
- [ ] Decision log records significant decisions and outcomes.

## OpenCode integration

- [ ] MCP discovery succeeds after restart.
- [ ] AIORG role/status/certificate tools are discoverable.
- [ ] OpenCode is treated as the host and AIORG as the enforcement boundary;
      duplicated agent state is not introduced.
