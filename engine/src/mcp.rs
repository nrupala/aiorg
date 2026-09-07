use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

/// Minimal MCP stdio server: JSON-RPC 2.0 over newline-delimited stdin/stdout.
/// Tools: aiorg_run, aiorg_status, aiorg_certificate_verify, aiorg_ask_role.
/// Results carry evidence summaries for zero-trust callers.
pub fn serve_stdio() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":format!("parse error: {e}")}});
                let _ = writeln!(stdout, "{}", resp);
                continue;
            }
        };
        let id = value.get("id").cloned().unwrap_or(Value::Null);
        let method = value.get("method").and_then(Value::as_str).unwrap_or("");
        let params = value.get("params").cloned().unwrap_or(json!({}));
        let response = match method {
            "initialize" => {
                json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"aiorg","version":"0.1.0"}}})
            }
            "tools/list" => json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
                {"name":"aiorg_run","description":"Run the AIORG delivery pipeline for a brief","inputSchema":{"type":"object","properties":{"brief":{"type":"string"},"project":{"type":"string"},"acceptance":{"type":"string"}},"required":["brief"]}},
                {"name":"aiorg_status","description":"List AIORG runs and verify a ledger chain","inputSchema":{"type":"object","properties":{"run_id":{"type":"string"},"project":{"type":"string"}},"required":[]}},
                {"name":"aiorg_certificate_verify","description":"Verify a run certificate","inputSchema":{"type":"object","properties":{"run_id":{"type":"string"},"project":{"type":"string"}},"required":["run_id"]}},
                {"name":"aiorg_ask_role","description":"Invoke a single AIORG role artifact","inputSchema":{"type":"object","properties":{"role":{"type":"string"},"project":{"type":"string"},"input":{"type":"string"}},"required":["role","input"]}}
            ]}}),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match name {
                    "aiorg_status" => {
                        let project = args.get("project").and_then(Value::as_str).unwrap_or(".");
                        match std::process::Command::new(
                            std::env::current_exe().unwrap_or_default(),
                        )
                        .arg("status")
                        .arg("--project")
                        .arg(project)
                        .output()
                        {
                            Ok(out) => {
                                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":String::from_utf8_lossy(&out.stdout).to_string()}]}})
                            }
                            Err(e) => {
                                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":e.to_string()}})
                            }
                        }
                    }
                    "aiorg_certificate_verify" => {
                        let run_id = args.get("run_id").and_then(Value::as_str).unwrap_or("");
                        let project = args.get("project").and_then(Value::as_str).unwrap_or(".");
                        match std::process::Command::new(
                            std::env::current_exe().unwrap_or_default(),
                        )
                        .arg("certificate")
                        .arg(run_id)
                        .arg("--project")
                        .arg(project)
                        .output()
                        {
                            Ok(out) => {
                                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":String::from_utf8_lossy(&out.stdout).to_string()}]}})
                            }
                            Err(e) => {
                                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":e.to_string()}})
                            }
                        }
                    }
                    "aiorg_run" | "aiorg_ask_role" => {
                        let exe = std::env::current_exe().unwrap_or_default();
                        let project = args.get("project").and_then(Value::as_str).unwrap_or(".");
                        let mu = json!({
                            "jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"run via `aiorg run --project <proj> --acceptance <cmd>` (async agents execute natively; run is not spawned here to avoid blocking stdio)"}]}});
                        match name {
                            "aiorg_run" => mu,
                            _ => {
                                let role = args
                                    .get("role")
                                    .and_then(Value::as_str)
                                    .unwrap_or("analyst");
                                let input = args.get("input").and_then(Value::as_str).unwrap_or("");
                                match std::process::Command::new(exe)
                                    .arg("ask")
                                    .arg(role)
                                    .arg("--project")
                                    .arg(project)
                                    .arg("--in")
                                    .arg(input)
                                    .output()
                                {
                                    Ok(out) => {
                                        json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":String::from_utf8_lossy(&out.stdout).to_string()}]}})
                                    }
                                    Err(e) => {
                                        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":e.to_string()}})
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("tool not found: {name}")}})
                    }
                }
            }
            "notifications/initialized" => continue,
            _ => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("method not found: {method}")}})
            }
        };
        let _ = writeln!(stdout, "{}", response);
    }
    Ok(())
}
