# 06 — Deployer Guide (AIORG v1)

Status: DRAFT for owner approval — validated step-by-step during build M7 on
this exact machine class. Every command below is meant to be run verbatim.

---

## 1. Prerequisites (this machine already satisfies all)

| Component | Required | Verify |
|---|---|---|
| Windows 11 + PowerShell 7 | ✔ | `$PSVersionTable.PSVersion` ≥ 7 |
| Rust MSVC toolchain (stable) + VS Build Tools | ✔ | `cargo --version`, `clippy` present |
| Node.js ≥ 22 (JS/TS gates) | ✔ | `node --version` |
| Python 3.12+ (pytest/ruff/mypy gates) | ✔ | `python --version` |
| CMake + MSVC or gcc/clang (C gates) | ✔ | `cmake --version` |
| Git | ✔ | `git --version` |
| WSL2 Ubuntu + bubblewrap + systemd (sandbox default) | ✔ installed; enable if fresh: `wsl -d Ubuntu -u root apt-get install -y bubblewrap systemd` | `wsl -l -v` shows Ubuntu; `wsl -d Ubuntu -- bwrap --version` |
| llama.cpp router preset | ✔ `C:\Users\nrupa\.config\opencode\local-engines\router-models.ini` | `llama-engine.ps1 -Status` |
| Ollama with pinned `-oc` variants | ✔ | `ollama list` shows `granite4:3b-oc`, `qwen3:1.7b-oc` |

**No Docker anywhere in this system. Do not install it for AIORG.**

## 2. Install

```powershell
# 2.1 Build (from D:\aiorg after build approval)
cargo build --release            # workspace → target\release\aiorg.exe
# optional: add to PATH
$env:Path += ";D:\aiorg\target\release"

# 2.2 First-run environment check
aiorg doctor
```

`aiorg doctor` verifies, in order: engines reachable (:8830 `/health`,
:11434 `/api/tags`) · expected model ids present · ports :8850/:8851 free ·
WSL+bwrap functional probe (`echo ok` inside a scoped sandbox, killed at 3 s)
· language toolchains for LX gates (node/python/cmake/cargo/sqlite/prettier —
missing ones are reported as *planning blockers per language*, not fatal)
· ledger chain integrity (after first run).

## 3. Bring up engines (if not already running)

```powershell
pwsh C:\Users\nrupa\.config\opencode\local-engines\llama-engine.ps1 -Start   # router :8830
# Ollama only if needed for fallback light roles:
pwsh C:\Users\nrupa\.config\opencode\local-engines\llama-engine.ps1 -OllamaStart
aiorg doctor    # must end with "environment: READY"
```

VRAM law (16 GB): AIORG never co-loads engines itself; the router swaps GGUFs
on demand. If you separately run Ollama with big models, expect router latency;
unload Ollama first (`-OllamaUnload` equivalent).

## 4. Configure

`D:\aiorg\aiorg.toml` (created by first `run`; defaults shown):

```toml
[engines]
router_url = "http://127.0.0.1:8830/v1"
ollama_url = "http://127.0.0.1:11434/v1"

[server]
port = 8850          # fail-fast if busy
mcp_http_port = 8851 # only when serve --mcp

[budgets]           # ceilings are policy; agents cannot raise them
max_cycles = 8
max_r = 3
stage_seconds = 600

[autonomy]
default = "L0"      # per-project overrides live in <project>/.aiorg/config.toml

[security]
run_key_env = "AIORG_RUN_KEY"
```

Secret handling:

```powershell
[Environment]::SetEnvironmentVariable("AIORG_RUN_KEY",
  [Convert]::ToBase64String((1..32 | % {Get-Random -Max 256}),0), "User")
# never paste this value into prompts, files, or tickets
```

## 5. First-run verification ladder (definition of "installed")

Run each; do not skip ahead on assumption:

1. `aiorg roles list` → prints 9 profiles with model routes.
2. `aiorg ask release --in .\README.md --dry-run` → schema-valid artifact from
   live router (proves provider+schema-retry path).
3. Sandbox probe: `aiorg doctor --sandbox` → `wsl scope: ok (killed@3000ms as designed)`.
4. Golden fixture: `aiorg run --project D:\aiorg\tmp-e2e "todo list CLI in JS,
   add/list/done"` → converges, certificate issued.
5. Tamper drill: edit any evidence file inside that run's artifacts, then
   `aiorg audit <RUN_ID>` → **must report chain mismatch** (zero-trust proven).
6. Replay drill: `aiorg replay <RUN_ID>` → identical certificate hash.

Only after 1–6 pass is the deployment called ready. Record outputs in your
ops log; the deployer guide is updated with any deviation found during M7.

## 6. Daily operation

```powershell
aiorg run "<brief>" --project <dir>        # full company
aiorg ask <role> --path <dir> ...          # single role on existing code
aiorg status                               # active runs table
aiorg resume <id> --answer "..."           # answer escalations
aiorg audit [id]                           # tamper-evident history
aiorg serve                                # SSE dashboard :8850 (+ MCP with --mcp)
```

Escalations appear in CLI and SSE; exit codes: 0 ok / 1 gate-fail /
2 escalated / 3 environment.

Artifacts of record live under `<project>/.aiorg/` — commit them like source
(they are the auditable truth). Central index DB stays in `D:\aiorg\data\`.

## 7. Troubleshooting

| Symptom | Cause → Fix |
|---|---|
| `doctor`: engine unreachable | Router not started → §3; wrong port → check `settings.json` of engine scripts |
| First response slow (~10 s) | Model swap on router → expected once per route change; keep Qwen3-8B resident |
| `policy.violation` event | Role touched out-of-scope path → inspect envelope; usually task scoping too tight → re-plan task |
| Sandbox probe fails | WSL stopped → `wsl -d Ubuntu -e true`; bwrap missing → §1 install line |
| Port :8850 busy | Another serve instance → `Get-NetTCPConnection -LocalPort 8850` then stop it |
| Schema retries exhausted often | Model too small for role → switch profile route up one tier in `org/roles/*.toml` |
| Gates report missing toolchain | Install listed tool (e.g., prettier for HTML/CSS/XML checks) or exclude language via policy |

## 8. Upgrade & uninstall

- Upgrade: `git pull && cargo build --release && aiorg doctor` (migrations are
  idempotent; ledger survives).
- Uninstall: delete `D:\aiorg`. Project-side `.aiorg/` dirs remain as historical
  records unless you remove them. No services are registered by default
  (`aiorg serve` runs only while invoked); autostart integration is opt-in v2.

## 9. Security posture notes

- All endpoints loopback-only; no inbound firewall holes.
- Run key: HMAC for certificates only; rotation = new env value (old certs stay
  verifiable via stored manifest hashes).
- Agents have no network tools; no cloud calls exist in the delivery path.
- Ledger is append-only + hash-chained; backups = copy `data\aiorg.db` plus the
  per-project `.aiorg/` folders.
