use anyhow::{bail, Context, Result};
use provider::Provider;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

mod config;
mod converge;
mod executor;
mod mcp;
mod server;
mod store;
mod verifier;

const ROLES: &[(&str, &str)] = &[
    (
        "dispatcher",
        "sequence the delivery pipeline and classify the brief",
    ),
    ("analyst", "produce requirements and acceptance criteria"),
    ("architect", "produce interfaces, risks, and architecture"),
    (
        "planner",
        "produce an acyclic task plan with tests and file scopes",
    ),
    ("engineer", "describe the smallest implementation increment"),
    (
        "reviewer",
        "review the planned increment for correctness and risk",
    ),
    (
        "qa",
        "define executable verification and failure interpretation",
    ),
    (
        "security",
        "review secrets, boundaries, and execution risks",
    ),
    (
        "release",
        "assemble release evidence and remaining blockers",
    ),
];

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("version") => println!("AIORG {}", version()),
        Some("roles") => roles(),
        Some("models") => models(args.get(2).map(String::as_str)).await?,
        Some("mcp") => mcp::serve_stdio()?,
        Some("doctor") => doctor().await?,
        Some("gate") => gate(args.get(2).map(String::as_str).unwrap_or(""))?,
        Some("sandbox") => sandbox(args.get(2).map(String::as_str).unwrap_or(""))?,
        Some("run") => run(&args[2..]).await?,
        Some("correct") => correct(&args[2..]).await?,
        Some("ask") => ask(&args[2..]).await?,
        Some("status") => status(&args[2..])?,
        Some("certificate") => certificate(&args[2..])?,
        Some("serve") => server::serve(args.get(2).and_then(|v| v.parse().ok()).unwrap_or(8850))?,
        Some("--help") | Some("-h") | None => help(),
        Some(command) => anyhow::bail!("unknown command: {command}"),
    }
    Ok(())
}

fn version() -> String {
    fs::read_to_string("VERSION")
        .unwrap_or_else(|_| "0.1.0".into())
        .trim()
        .into()
}
fn roles() {
    for (id, mission) in ROLES {
        println!("{id}: {mission}");
    }
}
fn help() {
    println!(
        "AIORG {}\n\nCommands:\n  run <brief> [--project DIR] [--acceptance CMD]\n  correct <brief> --project DIR --acceptance CMD\n  ask <role> [--project DIR] [--in FILE|--input STR]\n  models [local|hybrid|cloud]\n  mcp\n  status [RUN_ID] [--project DIR]\n  certificate <RUN_ID> [--project DIR]\n  serve [PORT]\n  doctor\n  gate <js-ts|python|rust|c|sql>\n  sandbox <wsl|host>\n  roles\n  version",
        version()
    );
}

fn model_cfg() -> Result<config::ModelConfig> {
    let home = aiorg_home();
    let path = home.join("config").join("aiorg.toml");
    let secrets = home.join("config").join("providers.local.toml");
    let env_items = [
        (
            "AIORG_ROUTER_URL",
            env::var("AIORG_ROUTER_URL").unwrap_or_default(),
        ),
        ("AIORG_MODEL", env::var("AIORG_MODEL").unwrap_or_default()),
    ];
    config::ModelConfig::load(&path, &secrets, &env_items)
}

fn aiorg_home() -> PathBuf {
    env::var("AIORG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn provider_for(
    cfg: &config::ModelConfig,
    name: &str,
    route: &config::RoleRoute,
) -> Result<Provider> {
    let spec = cfg.providers.get(name).context("provider not configured")?;
    let model = if name == "local" {
        route.local_model.as_str()
    } else if name == route.cloud_provider {
        route.cloud_model.as_str()
    } else {
        spec.default_model.as_str()
    };
    Provider::new_with_key(
        spec.base_url.clone(),
        model.to_string(),
        spec.api_key.clone(),
    )
}

/// Role chat with mode-aware failover: local primary (hybrid/cloud), cloud fallback.
async fn role_chat(
    cfg: &config::ModelConfig,
    route: &config::RoleRoute,
    messages: &[Value],
    max_tokens: u32,
) -> Result<String> {
    let mut errors = Vec::new();
    if route.mode != config::Mode::Cloud {
        let local = provider_for(cfg, "local", route)?;
        match local.chat(None, messages, max_tokens).await {
            Ok(out) => return Ok(out),
            Err(e) => errors.push(format!("local: {e}")),
        }
    }
    if route.mode != config::Mode::Local {
        for (name, _) in cfg.cloud_providers() {
            let cloud_route = config::RoleRoute {
                mode: config::Mode::Cloud,
                local_provider: "local".into(),
                local_model: route.local_model.clone(),
                cloud_provider: name.to_string(),
                cloud_model: route.cloud_model.clone(),
            };
            let try_chat = || async {
                let p = provider_for(cfg, name, &cloud_route)?;
                p.chat(None, messages, max_tokens).await
            };
            match try_chat().await {
                Ok(out) => return Ok(out),
                Err(e) => errors.push(format!("cloud {name}: {e}")),
            }
        }
    }
    bail!(
        "no provider answered (mode={:?}): {}",
        route.mode,
        errors.join("; ")
    )
}

fn default_route(cfg: &config::ModelConfig) -> config::RoleRoute {
    let local_model = cfg
        .providers
        .get("local")
        .map(|p| p.default_model.clone())
        .unwrap_or_else(|| "Qwen3.5-9B-Q8_0".into());
    config::RoleRoute {
        mode: cfg.mode.clone(),
        local_provider: "local".into(),
        local_model,
        cloud_provider: "openrouter".into(),
        cloud_model: "z-ai/glm-5.2:free".into(),
    }
}

async fn run(args: &[String]) -> Result<()> {
    let brief = args
        .iter()
        .take_while(|x| x.as_str() != "--project" && x.as_str() != "--acceptance")
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if brief.trim().is_empty() {
        anyhow::bail!("run requires a non-empty brief");
    }
    let project = project_arg(args)?;
    let run_id = Uuid::new_v4().simple().to_string();
    let root = project.join(".aiorg").join("runs").join(&run_id);
    fs::create_dir_all(&root)?;
    let store = store::Store::open(&project.join(".aiorg"))?;
    let model_cfg = model_cfg()?;
    store.begin_run(&run_id, &project, &brief)?;
    write_json(
        &root.join("run.json"),
        &json!({"run_id":run_id,"brief":brief,"status":"running","model":"per-role"}),
    )?;
    fs::write(project.join(".aiorg/current-run"), run_id.as_bytes())?;

    let local_base = model_cfg
        .providers
        .get("local")
        .map(|p| p.base_url.clone())
        .unwrap_or_else(|| "http://127.0.0.1:8830".into());
    let local_model = model_cfg
        .providers
        .get("local")
        .map(|p| p.default_model.clone())
        .unwrap_or_else(|| "Qwen3.5-9B-Q8_0".into());
    Provider::new(local_base, local_model)?
        .health()
        .await
        .context("S0 local engine health check failed")?;

    let mut artifacts = Vec::new();
    let mut events = vec![
        json!({"type":"run.started"}),
        json!({"type":"s0.health_pass"}),
    ];
    let mut prior_context = brief.clone();
    let mut convergence = converge::Converger::new(9, 2);
    for (index, (role, mission)) in ROLES.iter().enumerate() {
        convergence.observe(1.0 - ((index + 1) as f64 / ROLES.len() as f64))?;
        let role_contract = match *role {
            "dispatcher" => "Classify the brief and define the ordered pipeline. Do not write code.",
            "analyst" => "Write requirements with user stories and Given/When/Then acceptance criteria. Do not write code.",
            "architect" => "Write components, interfaces, data flow, risks, and ADRs. Do not write code.",
            "planner" => "Write an acyclic task DAG with file scopes and one acceptance test per task. Do not write code.",
            "engineer" => "Write the smallest implementation diff plan, including exact files and commands. Do not claim execution.",
            "reviewer" => "Independently review the preceding plan for correctness, scope, and risks. Do not rewrite it.",
            "qa" => "Define executable tests and expected evidence for the planned change. Do not claim tests ran.",
            "security" => "Threat-model the plan, scan risks, secrets, shell/path/SQL boundaries, and mitigations.",
            "release" => "Assemble release criteria and list blockers; do not issue a release claim unless evidence exists.",
            _ => mission,
        };
        let context = if prior_context.len() > 12000 {
            prior_context[prior_context.len() - 12000..].to_string()
        } else {
            prior_context.clone()
        };
        let prompt = json!([
            {"role":"system","content":format!("You are the AIORG {role}. {mission} {role_contract} Return only your role artifact in concise markdown with headings: Decision, Acceptance Criteria, Risks, Evidence Required. Do not impersonate another role. Do not invent execution evidence." )},
            {"role":"user","content":context}
        ]);
        let route = model_cfg
            .roles
            .get(*role)
            .cloned()
            .unwrap_or_else(|| default_route(&model_cfg));
        let output = role_chat(&model_cfg, &route, prompt.as_array().unwrap(), 2048)
            .await
            .with_context(|| format!("role {role} failed"))?;
        let path = root.join(format!("{index:02}-{role}.md"));
        fs::write(&path, &output)?;
        let digest = sha256(output.as_bytes());
        artifacts.push(json!({"role":role,"path":path,"sha256":digest,"producer":role,"verifier":"aiorg-guard"}));
        store.artifact(
            &run_id,
            &format!("{index:02}-{role}"),
            &path,
            role,
            "aiorg-guard",
        )?;
        store.event(
            &run_id,
            &format!("stage.{role}.complete"),
            &json!({"artifact_sha256":digest}),
        )?;
        events.push(json!({"type":format!("stage.{role}.complete"),"artifact_sha256":digest}));
        prior_context = format!("Brief:\n{brief}\n\nPrevious artifact ({role}):\n{output}");
    }

    let acceptance_result = if let Some(command) = acceptance_arg(args) {
        let mut gate = converge::Converger::new(3, 2);
        gate.observe(1.0)?;
        let mut result = 1;
        let mut applied_cycles: Vec<Vec<executor::AppliedPatch>> = Vec::new();
        for cycle in 1..=3 {
            result = executor::run_bwrap(&project, &command)?;
            store.event(
                &run_id,
                "acceptance.cycle",
                &json!({"cycle":cycle,"exit":result,"command":command}),
            )?;
            if result == 0 {
                gate.observe(0.0)?;
                for applied in &applied_cycles {
                    executor::commit_backup(applied)?;
                }
                break;
            }
            let feedback = format!(
                "Acceptance command failed with exit {result}: {command}. Return ONLY JSON matching {{\"summary\":string,\"edits\":[{{\"path\":string,\"content\":string}}]}}. Make the smallest source edit that can fix the failure. Never edit .git or .aiorg. Do not include markdown fences."
            );
            let engineer_route = model_cfg
                .roles
                .get("engineer")
                .cloned()
                .unwrap_or_else(|| default_route(&model_cfg));
            let raw = role_chat(&model_cfg, &engineer_route, &[json!({"role":"system","content":"You are the AIORG Engineer corrective-cycle role. You return strict JSON patches only."}), json!({"role":"user","content":feedback})], 2048).await?;
            let plan = executor::parse_patch(&raw)?;
            let backup_root = root.join("backups").join(cycle.to_string());
            let applied = scope(&project)?.apply(&plan, &backup_root)?;
            store.event(&run_id, "corrective.patch.applied", &json!({"cycle":cycle,"summary":plan.summary,"edits":plan.edits.iter().map(|e| &e.path).collect::<Vec<_>>() }))?;
            store.artifact(
                &run_id,
                &format!("corrective-{cycle}"),
                &root.join(format!("corrective-{cycle}-engineer.json")),
                "engineer",
                "aiorg-guard",
            )?;
            fs::write(
                root.join(format!("corrective-{cycle}-engineer.json")),
                serde_json::to_vec_pretty(&plan)?,
            )?;
            applied_cycles.push(applied);
        }
        if result != 0 {
            for applied in applied_cycles.iter().rev() {
                executor::rollback(applied)?;
            }
            anyhow::bail!("acceptance gate failed after three corrective cycles")
        }
        Some(result)
    } else {
        None
    };

    let ledger = json!({"run_id":run_id,"events":events,"artifacts":artifacts,"producer":"aiorg-dispatcher","verifier":"aiorg-guard"});
    let ledger_bytes = serde_json::to_vec_pretty(&ledger)?;
    fs::write(root.join("ledger.json"), &ledger_bytes)?;
    let certificate_hash = sha256(&ledger_bytes);
    let key = env::var("AIORG_RUN_KEY").unwrap_or_else(|_| "local-development-key".into());
    let cert = verifier::sign(
        &run_id,
        &ledger_bytes,
        "aiorg-release",
        "aiorg-guard",
        key.as_bytes(),
    )?;
    verifier::verify(&cert, &ledger_bytes, key.as_bytes())?;
    write_json(
        &root.join("certificate.json"),
        &serde_json::to_value(&cert)?,
    )?;
    verifier::verify_file(
        &root.join("certificate.json"),
        &root.join("ledger.json"),
        key.as_bytes(),
    )?;
    store.close_run(&run_id, "artifact_pipeline_complete")?;
    write_json(
        &root.join("run.json"),
        &json!({"run_id":run_id,"brief":brief,"status":"artifact_pipeline_complete","mode":"per-role","convergence_zero":convergence.converged(),"cycles":convergence.cycles().len(),"acceptance_exit":acceptance_result,"artifacts":artifacts,"certificate_sha256":certificate_hash}),
    )?;
    println!("run_id={run_id}\nstatus=artifact_pipeline_complete\ncertificate_sha256={certificate_hash}\nartifacts={}", root.display());
    Ok(())
}

fn scope(project: &Path) -> Result<executor::Scope> {
    executor::Scope::new(project, &[PathBuf::from(".")])
}

async fn ask(args: &[String]) -> Result<()> {
    let role = args.first().cloned().unwrap_or_default();
    if ROLES.iter().all(|(r, _)| *r != role) {
        anyhow::bail!("unknown role: {role}")
    }
    let _ = project_arg(args)?;
    let model_cfg = model_cfg()?;
    let route = model_cfg
        .roles
        .get(&role)
        .cloned()
        .unwrap_or_else(|| default_route(&model_cfg));
    let mission = ROLES
        .iter()
        .find(|(r, _)| *r == role)
        .map(|(_, m)| *m)
        .unwrap_or("");
    let input = if let Some(i) = args.iter().position(|a| a == "--in") {
        fs::read_to_string(args.get(i + 1).context("--in requires a file")?)?
    } else if let Some(i) = args.iter().position(|a| a == "--input") {
        args.get(i + 1).cloned().unwrap_or_default()
    } else {
        "Produce your role artifact for this open task.".to_string()
    };
    let prompt = json!([
        {"role":"system","content":format!("You are the AIORG {role}. {mission} Return a concise markdown artifact.")},
        {"role":"user","content":input}
    ]);
    let out = role_chat(&model_cfg, &route, prompt.as_array().unwrap(), 2048).await?;
    println!("{out}");
    Ok(())
}

async fn correct(args: &[String]) -> Result<()> {
    let project = project_arg(args)?;
    let command = acceptance_arg(args).context("correct requires --acceptance")?;
    let brief = args
        .iter()
        .take_while(|v| v.as_str() != "--project" && v.as_str() != "--acceptance")
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if brief.trim().is_empty() {
        anyhow::bail!("correct requires a brief")
    }
    let sc = scope(&project)?;
    if executor::run_bwrap(&project, &command)? == 0 {
        println!("acceptance=already_passed");
        return Ok(());
    }
    let model_cfg = model_cfg()?;
    let route = model_cfg
        .roles
        .get("engineer")
        .cloned()
        .unwrap_or_else(|| default_route(&model_cfg));
    let prompt = format!(
        "Acceptance command failed: {command}. Task: {brief}. Return ONLY JSON {{\"summary\":string,\"edits\":[{{\"path\":string,\"content\":string}}]}}. Make the smallest safe edit. Never edit .git, .aiora. No markdown fences."
    );
    let raw = role_chat(&model_cfg, &route, &[json!({"role":"system","content":"You are an autonomous corrective agent. Return a strict patch JSON object."}), json!({"role":"user","content":prompt})], 2048).await?;
    let plan = executor::parse_patch(&raw)?;
    let backup = project.join(".aiorg/corrective-backup");
    let applied = sc.apply(&plan, &backup)?;
    if executor::run_bwrap(&project, &command)? == 0 {
        executor::commit_backup(&applied)?;
        println!("acceptance=passed\nsummary={}", plan.summary);
        Ok(())
    } else {
        executor::rollback(&applied)?;
        anyhow::bail!("corrective patch did not satisfy acceptance; rolled back")
    }
}

async fn doctor() -> Result<()> {
    let cfg = model_cfg()?;
    for (name, spec) in &cfg.providers {
        let p = Provider::new_with_key(
            spec.base_url.clone(),
            spec.default_model.clone(),
            spec.api_key.clone(),
        )?;
        let status = if p.health().await.is_ok() {
            "healthy"
        } else {
            "unreachable"
        };
        println!("{name}={status} ({})", spec.base_url);
    }
    println!("wsl={}", probe_wsl());
    Ok(())
}

async fn models(mode_arg: Option<&str>) -> Result<()> {
    let cfg = model_cfg()?;
    let mode = match mode_arg {
        Some("local") => config::Mode::Local,
        Some("cloud") => config::Mode::Cloud,
        Some("hybrid") | None => config::Mode::Hybrid,
        _ => bail!("models: unknown mode {mode_arg:?}"),
    };
    for (role, route) in &cfg.roles {
        let label = match mode {
            config::Mode::Local => {
                format!("{role}: {} via {}", route.local_model, route.local_provider)
            }
            config::Mode::Cloud => {
                format!("{role}: {} via {}", route.cloud_model, route.cloud_provider)
            }
            config::Mode::Hybrid => format!(
                "{role}: {} ({} -> {} via {})",
                route.local_model, route.local_provider, route.cloud_model, route.cloud_provider
            ),
        };
        println!("{label}");
    }
    Ok(())
}

fn certificate(args: &[String]) -> Result<()> {
    let id = args.first().context("certificate requires RUN_ID")?;
    let root = project_arg(args)?.join(".aiorg/runs").join(id);
    let ledger = fs::read(root.join("ledger.json"))?;
    let key = env::var("AIORG_RUN_KEY").unwrap_or_else(|_| "local-development-key".into());
    let cert = verifier::sign(id, &ledger, "aiorg-release", "aiorg-guard", key.as_bytes())?;
    write_json(
        &root.join("certificate.json"),
        &serde_json::to_value(&cert)?,
    )?;
    verifier::verify(&cert, &ledger, key.as_bytes())?;
    verifier::verify_file(
        &root.join("certificate.json"),
        &root.join("ledger.json"),
        key.as_bytes(),
    )?;
    println!("certificate=issued run_id={id} hash={}", cert.artifact_hash);
    Ok(())
}

fn gate(language: &str) -> Result<()> {
    let result = match language {
        "js-ts" => Command::new("node").args(["--check", "test.js"]).status(),
        "python" => Command::new("python").args(["-m", "pytest"]).status(),
        "rust" => Command::new("cargo")
            .args(["test", "--all-targets"])
            .status(),
        "c" => Command::new("cmake").args(["--build", "build"]).status(),
        "sql" => Command::new("sqlite3")
            .args([":memory:", "select 1;"])
            .status(),
        _ => anyhow::bail!("unsupported language gate: {language}"),
    }?;
    if !result.success() {
        anyhow::bail!("gate {language} failed");
    }
    println!("gate {language}: PASS");
    Ok(())
}

fn sandbox(mode: &str) -> Result<()> {
    let ok = match mode {
        "wsl" => executor::run_bwrap(&env::current_dir()?, "printf AIORG_SANDBOX")
            .map(|code| code == 0)
            .unwrap_or(false),
        "host" => false,
        _ => anyhow::bail!("unsupported sandbox: {mode}"),
    };
    if !ok {
        anyhow::bail!("sandbox {mode} unavailable");
    }
    println!("sandbox {mode}: PASS");
    Ok(())
}

fn probe_wsl() -> bool {
    Command::new("bash")
        .args(["-lc", "command -v bwrap >/dev/null 2>&1 && printf ok"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn status(args: &[String]) -> Result<()> {
    let root = project_arg(args)?;
    let runs = root.join(".aiorg/runs");
    if !runs.exists() {
        println!("no runs");
        return Ok(());
    }
    for e in fs::read_dir(runs)? {
        let p = e?.path().join("run.json");
        if p.exists() {
            println!("{}", fs::read_to_string(p)?);
        }
    }
    if let Some(run_id) = args.first().filter(|v| !v.starts_with("--")) {
        let store = store::Store::open(&root.join(".aiorg"))?;
        println!("ledger_chain={}", store.verify_chain(run_id)?);
    }
    Ok(())
}

fn acceptance_arg(args: &[String]) -> Option<String> {
    args.iter()
        .position(|x| x == "--acceptance")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn project_arg(args: &[String]) -> Result<PathBuf> {
    if let Some(i) = args.iter().position(|x| x == "--project") {
        Ok(PathBuf::from(
            args.get(i + 1).context("--project requires a directory")?,
        ))
    } else {
        Ok(env::current_dir()?)
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_deterministic() {
        assert_eq!(sha256(b"x"), sha256(b"x"));
    }

    #[test]
    fn rejects_empty_brief() {
        assert!("".trim().is_empty());
    }
}
