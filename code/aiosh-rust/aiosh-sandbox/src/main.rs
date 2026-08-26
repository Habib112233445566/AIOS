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
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: aiosh-sandbox --policy <json> -- <bin> <args...>";

    // Parse `--policy <json> -- <bin> <args...>` without panicking on
    // truncated argv (`--policy` as the final argument, etc.).
    let mut policy_json = "{}".to_string();
    let mut argv: Vec<String> = Vec::new();
    match args.iter().position(|a| a == "--policy") {
        Some(i) => {
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
            policy_json = raw;
            argv = rest[1..].to_vec();
        }
        None => {
            // No policy: everything after `--` is the command.
            if args.first().map(|s| s.as_str()) != Some("--") {
                eprintln!("{usage}");
                return ExitCode::from(2);
            }
            argv = args[1..].to_vec();
        }
    }

    if argv.is_empty() {
        eprintln!("sandbox: empty argv");
        return ExitCode::from(2);
    }

    let policy = match SandboxPolicy::from_json(&policy_json) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("sandbox: invalid policy: {}", e);
            return ExitCode::from(2);
        }
    };

    let code = sandbox_exec(&argv, &policy);
    ExitCode::from(code as u8)
}
