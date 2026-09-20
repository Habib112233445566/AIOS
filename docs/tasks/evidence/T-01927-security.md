# Task Evidence: T-01927 - System Update / CLI surface: Security Review

- **Task**: `T-01927`
- **Sub-Epic**: `Sub-Epic 3: Operator CLI & Control Surface`
- **Status**: Completed
- **Date**: 2026-09-20

## Threat Modeling & Security Analysis (THREAT-UCLI-01..06)

### 1. THREAT-UCLI-01: Path Traversal / Injection via Flags
- **Vector**: Operator passes malicious paths (e.g., control characters, oversized buffers, relative traversal) via `--state-dir` or `--staging-dir`.
- **Impact**: Arbitrary file overwrite, reading outside permitted directories, or memory denial of service.
- **Mitigation**:
  - Bound path strings to `MAX_PATH_LEN = 1024`.
  - Reject control characters (`\n`, `\t`, `\r`, `\0`, etc.) immediately with exit code 2 and structured error code `PATH_CONTAINS_CONTROL_CHAR`.
  - Record security failure audit event before process exit.

### 2. THREAT-UCLI-02: Positional Argument Confusion & Injection
- **Vector**: Arguments provided after options or interleaved with flags could misroute operational parameters (e.g., a path argument treated as a version string).
- **Impact**: Unintended version confirmation or wrong file reading.
- **Mitigation**: Dedicated `extract_update_positional_args` parser filters out recognized option flags (`--state-dir`, `--staging-dir`, `--version`, `--slot`) and their corresponding values before binding positional operands.

### 3. THREAT-UCLI-03: Terminal ANSI Escape Injection
- **Vector**: Stored or returned version strings, error messages, or filenames contain malicious ANSI escape sequences targeting operator terminal emulators.
- **Impact**: Terminal hijacking, window title spoofing, or command obfuscation.
- **Mitigation**: Every dynamic string printed to stdout/stderr in human-readable mode is filtered through `sanitize_terminal()`, stripping non-printable and escape byte sequences.

### 4. THREAT-UCLI-04: Non-Deterministic Exit Codes
- **Vector**: Automation pipelines and orchestrators misinterpret failure states if exit codes are non-standard.
- **Impact**: Failed update treated as success, leading to unverified boots or bad slot deployment.
- **Mitigation**: Enforce strict exit code contract:
  - `0`: Operation succeeded cleanly.
  - `1`: Domain or operational failure (e.g., manifest check failed, invalid state transition).
  - `2`: CLI argument error, unknown subcommand, or security path hygiene rejection.

### 5. THREAT-UCLI-05: Unaudited Operator Mutations
- **Vector**: Privileged users execute `apply`, `confirm`, or `rollback` without leaving forensic evidence.
- **Impact**: Untraceable system state changes and impaired incident response.
- **Mitigation**: Every CLI command invocation emits an audit record via `classify_and_emit` into the local SQLite WAL audit ring with command name, arguments, status, and actor.

### 6. THREAT-UCLI-06: Atomic Persistence & State Race Conditions
- **Vector**: Concurrent CLI invocations corrupting JSON state files during write.
- **Impact**: Corrupted `slot_status.json` or `update_status.json` causing unbootable target configuration.
- **Mitigation**: State updates are performed atomically via temporary files (`.tmp`) and atomic file replacement (`fs::rename`), with validation on reload.

## Verdict
Threat model verified and security constraints documented. Ready for hardening verification and documentation closure.
