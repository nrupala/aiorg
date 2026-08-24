/// AIORG — Orchestrated AI Software Development Company
/// M1 scaffolding: version display + role listing.
use std::fs;
use std::path::Path;

fn main() {
    // Read VERSION file (canonical source of truth)
    let version = fs::read_to_string("VERSION")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "0.1.0".to_string());

    // If no VERSION beside binary, try sibling D:\aiorg\VERSION
    let version_path = Path::new("VERSION");
    if !version_path.exists() {
        let alt = Path::new("D:/aiorg/VERSION");
        if alt.exists() {
            let s = fs::read_to_string(alt).unwrap().trim().to_string();
            println!("AIORG version: {}\n(loaded from D:/aiorg/VERSION)", s);
        } else {
            println!("AIORG version: {}", version);
        }
    } else {
        println!("AIORG version: {}", version);
    }

    // List the 9 v1 roles from org/roles TOML files (hardcoded for M1)
    let roles: &[&str] = &[
        "Dispatcher: Qwen3-8B-Q4_K_M",
        "Analyst: Qwen3-8B",
        "Architect: DeepSeek-R1-0528-Qwen3-8B",
        "Planner: Qwen3-8B",
        "Engineer: qwen2.5-coder-7b-instruct-q4_k_m",
        "Reviewer: Qwen3-8B",
        "QA: qwen2.5-coder-7b-instruct-q4_k_m",
        "Security: DeepSeek-R1-0528-Qwen3-8B",
        "Release Manager: Qwen3-8B",
    ];

    println!("\nAIORG Roles (v1 core):");
    for (i, role) in roles.iter().enumerate() {
        println!("{}. {}", i + 1, role);
    }

    // Simple doctor probe: check if llama.cpp router is reachable on :8830
    print!("\nEngine probe: ");
    match std::net::TcpStream::connect("127.0.0.1:8830") {
        Ok(_) => println!("llama.cpp router :8830 reachable"),
        Err(_) => println!("llama.cpp router :8830 not reachable (start with llama-engine.ps1 -Start)"),
    }

    // Show CARGO_HOME / project root for debugging
    println!("\nRun `aiorg --help` for command list (M1 skeleton).");
}