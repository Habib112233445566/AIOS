# T-00537 — Evidence & Audit Trail / CLI surface: Security Review

## 1. Overview
This security review evaluates the command-line interface surface for Evidence & Audit Trail (`aiosh evidence <verify|hash|scan>`), analyzing argument parsing, output escaping, filesystem path boundaries, and audit trail fidelity.

## 2. Threat Scenarios & Mitigations

### A. Terminal Escape & ANSI Injection
- **Threat**: Maliciously named files or malformed markdown headers injecting terminal control escape characters into operator stdout.
- **Evaluation**: The CLI enforces structured formatting and standard JSON serialization, preventing raw binary/control code injection into terminal emulators.

### B. Argument Smuggling & Positional Ambiguity
- **Threat**: Injecting nested flags or unparsed positional arguments to divert execution.
- **Evaluation**: The CLI uses deterministic flag extraction (`parse_flag`, `has_flag`, `strip_flags`) and fails with exit code 2 on missing or ambiguous arguments.

### C. Path Traversal via `--manifest` or `--repo`
- **Threat**: Supplying arbitrary filesystem locations to bypass repository root boundaries.
- **Evaluation**: Core verification routines in `evidence_service` assert relative path containment and 16 MiB size bounds on all inspected files.

### D. Audit Ring Integrity
- **Threat**: Suppressing audit event logging for evidence verification.
- **Evaluation**: All invocations route through `emit(&mut ctx, ...)` which appends an immutable row with deterministic SHA-256 hash to SQLite WAL.

## 3. Findings & Verdict
The CLI surface is secure, enforces strict argument parsing, sanitizes outputs, and logs all execution events. No policy bypasses exist.
