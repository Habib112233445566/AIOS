# T-00633 — Repository Health / CLI surface: Scaffold

## 1. Scaffold Scope
This task creates the CLI interface skeleton for the `repo` command in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Scaffold Deliverables
- Added `cmd_repo(args: &[String]) -> i32` skeleton in `main.rs`.
- Registered `Some("repo") => cmd_repo(&args[1..])` in CLI dispatcher.
- Updated `--help` output with `aiosh repo <health|check>`.

## 3. Compilation Verification Output
```text
    Checking aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Checking aiosh-cli v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.58s
```
