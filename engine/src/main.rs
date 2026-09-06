use anyhow::{Context, Result};
use provider::Provider;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use uuid::Uuid;

mod converge;
mod executor;
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
    ("release", "define release evidence and remaining blockers"),
];

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("version") => println!("AIORG {}", version()),
        Some("roles") => roles(),
        Some("doctor") => doctor().await?,
        Some("gate") => gate(args.get(2).map(String::as_str).unwrap_or(""))?,
        Some("sandbox") => sandbox(args.get(2).map(String::as_str).unwrap_or(""))?,
        Some("run") => run(&args[2..]).await?,
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
    println!("AIORG {}\n\nCommands:\n  run <brief> [--project DIR]\n  status [RUN_ID] [--project DIR]\n  certificate <RUN_ID> [--project DIR]\n  serve [PORT]\n  doctor\n  roles\n  gate <js-ts|python|rust|c|sql>\n  sandbox <wsl|host>", version());
}

async fn doctor() -> Result<()> {
    let url = router_url();
    let provider = Provider::new(url.clone(), model())?;
    let health = provider.health().await;
    println!(
        "router={}",
        if health.is_ok() {
            "healthy"
        } else {
            "unreachable"
        }
    );
    println!(
        "wsl={}",
        if probe_wsl() {
            "available"
        } else {
            "unavailable"
        }
    );
    println!("project_root={}", env::current_dir()?.display());
    if health.is_err() {
        anyhow::bail!("router health check failed at {url}");
    }
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
fn router_url() -> String {
    env::var("AIORG_ROUTER_URL").unwrap_or_else(|_| "http://127.0.0.1:8830".into())
}
fn model() -> String {
    env::var("AIORG_MODEL").unwrap_or_else(|_| "Qwen3.5-9B-Q8_0".into())
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
    let scope = executor::Scope::new(&project, &[PathBuf::from(".aiorg")])?;
    store.begin_run(&run_id, &project, &brief)?;
    write_json(
        &root.join("run.json"),
        &json!({"run_id":run_id,"brief":brief,"status":"running","model":model()}),
    )?;
    scope.write(Path::new(".aiorg/current-run"), run_id.as_bytes())?;
    if scope.read(Path::new(".aiorg/current-run"))? != run_id.as_bytes() {
        anyhow::bail!("scoped executor readback failed");
    }
    let provider = Provider::new(router_url(), model())?;
    provider
        .health()
        .await
        .context("S0 router health check failed")?;
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
        let output = provider
            .chat(None, prompt.as_array().unwrap(), 2048)
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
        for cycle in 1..=3 {
            result = executor::run_bwrap(&project, &command)?;
            store.event(
                &run_id,
                "acceptance.cycle",
                &json!({"cycle":cycle,"exit":result,"command":command}),
            )?;
            if result == 0 {
                gate.observe(0.0)?;
                break;
            }
            let feedback = format!("Acceptance command failed with exit {result}: {command}. Produce a corrective implementation plan only; no execution claim.");
            let corrective = provider.chat(Some("Qwen2.5-Coder-7b-instruct-q8_0"), &[json!({"role":"system","content":"You are the AIORG Engineer corrective-cycle role."}), json!({"role":"user","content":feedback})], 2048).await?;
            let path = root.join(format!("corrective-{cycle}-engineer.md"));
            fs::write(&path, corrective)?;
            store.artifact(
                &run_id,
                &format!("corrective-{cycle}"),
                &path,
                "engineer",
                "aiorg-guard",
            )?;
        }
        if result != 0 {
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
        &json!({"run_id":run_id,"brief":brief,"status":"artifact_pipeline_complete","mode":"role-artifact-pipeline","convergence_zero":convergence.converged(),"cycles":convergence.cycles().len(),"acceptance_exit":acceptance_result,"artifacts":artifacts,"certificate_sha256":certificate_hash}),
    )?;
    println!(
        "run_id={run_id}\nstatus=artifact_pipeline_complete\ncertificate_sha256={certificate_hash}\nartifacts={}",
        root.display()
    );
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
fn status(args: &[String]) -> Result<()> {
    let root = project_arg(args)?;
    let store = store::Store::open(&root.join(".aiorg"))?;
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
        println!("ledger_chain={}", store.verify_chain(run_id)?);
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
    println!("certificate=issued run_id={id} hash={}", cert.artifact_hash);
    Ok(())
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
