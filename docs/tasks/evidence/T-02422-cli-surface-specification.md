# T-02422: Sandbox Enforcement CLI Surface Specification

## 1. Specification Overview
This specification defines the syntax, options, input validation, execution behavior, error handling, exit codes, and audit trail effects for the **Sandbox Enforcement CLI Subcommand** (`aiosh sandbox` / `aiosh sb`) in `code/aiosh-rust/aiosh-cli`.

---

## 2. Command Synopsis & Subcommands

```text
aiosh sandbox <subcommand> [options]

Subcommands:
  profiles [--json]                       List registered sandbox containment profiles
  probe [--json]                          Probe and report host kernel sandbox capabilities
  exec [--profile <P>] [--grant <G>] [--cwd <D>] -- <cmd...>
                                          Execute a command under sandbox containment
```

### A. Subcommand: `profiles`
- **Options**:
  - `--json`: Output as raw JSON array of `SandboxProfile` objects.
- **Output (Human-Readable)**:
  - Tabular display of profile name, isolation level, network mode, and memory limits.
- **Exit Code**: `0` on success.

### B. Subcommand: `probe`
- **Options**:
  - `--json`: Output as raw JSON object of `HostSandboxCapabilities`.
- **Output (Human-Readable)**:
  - Summary of Landlock LSM availability, ABI version, Seccomp-BPF support, `no_new_privs` support, and host platform.
- **Exit Code**: `0` on success.

### C. Subcommand: `exec`
- **Options**:
  - `--profile <name>`: Target profile name (default: `"standard"`).
  - `--grant <token>`: PEP grant capability token (required if PEP enforcement enabled).
  - `--cwd <path>`: Working directory for process execution. Must not contain directory traversal sequences (`..`).
  - `--`: Mandatory delimiter separating `aiosh` flags from the sandboxed binary and its arguments.
- **Output**:
  - Pipes child process stdout and stderr directly to console streams.
- **Exit Code**:
  - Returns the child process exit code (e.g., `0` for success, non-zero for failure).
  - Returns `2` on CLI option parsing errors or invalid parameters.
  - Returns `127` if the target executable was not found.
  - Returns `1` if PEP authorization is denied.

---

## 3. Audit Logging Invariant
Every invocation of `aiosh sandbox exec` writes an immutable record to the SQLite WAL `audit_ring` table:
- `tool`: `"sandbox"`
- `command`: Target binary path
- `args`: JSON object containing arguments, profile name, duration, and exit code
- `outcome`: `"ok"` if child exited with 0, `"error"` otherwise
- `grant_token`: Optional PEP grant token from `--grant`
