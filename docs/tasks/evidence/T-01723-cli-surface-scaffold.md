# T-01723: Hardware Detection — CLI Surface Scaffold

## Metadata
- **Task ID**: `T-01723`
- **Sub-Epic**: Sub-Epic 3: Hardware Detection CLI Surface
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Scaffolding Scope
1. Registered `aiosh hw` and `aiosh hardware` aliases in `code/aiosh-rust/aiosh-cli/src/main.rs`:
   - Updated top-level command matching in `fn main()`.
   - Updated usage text for `--help`.
2. Created module handler skeleton `fn cmd_hardware(args: &[String]) -> i32`:
   - Dispatches subcommands `scan`, `list`, `show`, `summary`, and `verify`.
   - Supports `--help` / `-h`.
   - Rejects unknown subcommands with exit code 2 and standard error envelope.
   - Emits audit rows to local SQLite WAL ring.

---

## 2. Compilation Verification
Command:
```bash
cargo check -p aiosh-cli
```
Output:
```
Checking aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
Checking aiosh-cli v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-cli)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.96s
```
Status: **PASS (0 errors, 0 warnings)**.
