//! AIOS sandbox CLI — standalone Landlock + seccomp-bpf executor.
//!
//! Drop-in replacement for the legacy `python -m aiosh_mcp.sandbox`
//! interface, so the TypeScript CLI (`aiosh run`) and any other caller
//! can sandbox a host command **without the Python package installed**:
//!
//! ```text
//! aiosh-sandbox --policy <json> -- <bin> <args...>
//! ```
//!
//! Behaviour mirrors `sandbox.py`:
//!   - fork; in the child apply no_new_privs → seccomp blacklist →
//!     Landlock rules, then `execve` the command;
//!   - the child emits a one-line `{"event":"sandbox_applied",
//!     "components":[[name,status],...]}` JSON to stderr before execve
//!     (the parent/CLI parses it for the audit row);
//!   - the parent reaps the child and returns its exit code (128+sig on
//!     signal death, e.g. a seccomp kill).

use aiosh_core::sandbox::{sandbox_exec, SandboxPolicy};
use aiosh_core::sandbox_data_model::SandboxProfile;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: aiosh-sandbox [--policy <json> | --profile <name_or_json>] -- <bin> <args...>";

    // Parse `--profile <arg>` or `--policy <json> -- <bin> <args...>`
    let profile_pos = args.iter().position(|a| a == "--profile");
    let policy_pos = args.iter().position(|a| a == "--policy");

    let (policy, argv) = if let Some(i) = profile_pos {
        let raw = match args.get(i + 1) {
            Some(v) => v.clone(),
            None => {
                eprintln!("{usage}");
                return ExitCode::from(2);
            }
        };
        let rest = &args[(i + 2).min(args.len())..];
        if rest.first().map(|s| s.as_str()) != Some("--") {
            eprintln!("{usage}");
            return ExitCode::from(2);
        }
        let pol = match raw.to_lowercase().as_str() {
            "standard" => SandboxProfile::standard().to_legacy_policy(),
            "strict" => SandboxProfile::strict().to_legacy_policy(),
            "permissive" => SandboxProfile::permissive().to_legacy_policy(),
            _ => match serde_json::from_str::<SandboxProfile>(&raw) {
                Ok(p) => p.to_legacy_policy(),
                Err(e) => {
                    eprintln!("sandbox: invalid profile JSON: {}", e);
                    return ExitCode::from(2);
                }
            },
        };
        (pol, rest[1..].to_vec())
    } else if let Some(i) = policy_pos {
        let raw = match args.get(i + 1) {
            Some(v) => v.clone(),
            None => {
                eprintln!("{usage}");
                return ExitCode::from(2);
            }
        };
        let rest = &args[(i + 2).min(args.len())..];
        if rest.first().map(|s| s.as_str()) != Some("--") {
            eprintln!("{usage}");
            return ExitCode::from(2);
        }
        let pol = if let Ok(p) = serde_json::from_str::<SandboxProfile>(&raw) {
            p.to_legacy_policy()
        } else {
            match SandboxPolicy::from_json(&raw) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("sandbox: invalid policy: {}", e);
                    return ExitCode::from(2);
                }
            }
        };
        (pol, rest[1..].to_vec())
    } else {
        // No policy/profile: everything after `--` is the command.
        if args.first().map(|s| s.as_str()) != Some("--") {
            eprintln!("{usage}");
            return ExitCode::from(2);
        }
        (SandboxPolicy::default(), args[1..].to_vec())
    };

    if argv.is_empty() {
        eprintln!("sandbox: empty argv");
        return ExitCode::from(2);
    }

    let code = sandbox_exec(&argv, &policy);
    ExitCode::from(code as u8)
}
