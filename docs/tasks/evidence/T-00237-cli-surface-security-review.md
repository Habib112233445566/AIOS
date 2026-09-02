# T-00237 — Phase 0 — Release Packaging & Backup / CLI surface: Security Review

## Goal
Security-review the CLI surface of Release Packaging & Backup.

## Abuse Scenarios Evaluated

### 1. Argument / Path Injection
**Scenario**: The user inputs malicious characters in `--target-path`, `--os`, or `--version` (e.g., `../../etc/passwd` or `| bash`).
**Evaluation**: The CLI does no execution of shell commands, and simply passes string references directly into the `PackageManifest` and `BackupSnapshot` Rust structs. Subprocess injection is impossible because `aiosh-core`'s physical I/O layer handles file logic natively without invoking `sh -c`. Path traversal is defended at the `aiosh-core` boundaries via path sanitization implemented in earlier tasks.
**Status**: Mitigated natively.

### 2. Audit Evasion via Pre-Emptive Failure
**Scenario**: An attacker issues `aiosh backup create` with missing arguments (e.g., missing `--target-path`) to see if they can trigger a crash before the CLI audits the failure.
**Evaluation**: Missing CLI arguments correctly return a local exit code `2` with a usage string. As per `aiosh` conventions, syntax-level rejections at the outer CLI frame are unauthenticated noise and not required to hit the `AuditRing`. Only once a command is structurally valid and invokes `aiosh_core::release::create_backup` is it fully audited (including its runtime errors).
**Status**: Expected behavior.

### 3. Untrusted Memory Toggle
**Scenario**: Exploiting the `--include-memory` flag to dump privileged memory.
**Evaluation**: The flag simply sets a boolean on the `BackupSnapshot` struct. The core logic enforces permissions on what can and cannot be accessed during backup operations.
**Status**: Mitigated centrally.

## Verification of Invariants
- **PEP Gating**: Handled identically to the core modules.
- **Audit-Row Emission**: `cmd_release` and `cmd_backup` inject `ReleaseCtx` (carrying a mutable `AuditRing` reference) directly into `aiosh_core::release`. Thus, exactly one state-changing DB row is emitted per valid command invocation, fulfilling ADR-0035 flawlessly. 

## Acceptance Criteria Verified
- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open.
