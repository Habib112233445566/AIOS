# T-02423: Sandbox Enforcement CLI Surface Scaffold

## 1. Scaffold Summary
This document provides verification evidence for task T-02423: Sandbox Enforcement CLI Surface Scaffold.
The `aiosh sandbox` (and alias `aiosh sb`) command router and handler function `cmd_sandbox(args: &[String]) -> i32` have been wired into `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Command Architecture & Routing
The CLI entrypoint `main()` routes invocations:
```rust
Some("sandbox") | Some("sb") => cmd_sandbox(&args[1..]),
```

`cmd_sandbox` defines typed command handling for all sandbox subcommands:
1. `profiles`:
   - Inspects registered sandbox containment profiles from `SandboxService`.
   - Supports `--json` envelope output and tabular human-readable output.
2. `probe`:
   - Probes kernel containment capabilities via `HostSandboxCapabilities::probe()`.
   - Supports `--json` envelope output and human-readable reporting of Landlock ABI version, Seccomp-BPF support, and `no_new_privs`.
3. `exec`:
   - Enforces `--` delimiter separation between `aiosh` flags and execution command.
   - Enforces directory traversal hygiene on `--cwd` (`..` checks).
   - Validates profile presence against `SandboxService` catalog.
   - Dispatches execution to `SandboxService::execute`.
4. `--help` / `-h` / None:
   - Formats help screen and returns exit code 0.
5. Unknown subcommand:
   - Emits audit failure event and returns exit code 2.

## 3. Build & Compilation Verification
`cargo check --workspace` and `cargo check -p aiosh-cli` completed cleanly with zero warnings and zero errors:
```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.44s
```
