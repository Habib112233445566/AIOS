# Task Completion Evidence: T-01622

## Task Overview
- **Task ID**: T-01622
- **Task Name**: Phase 1 — Linux Base System & Bootable Target / Kernel Module Management / CLI surface: Specification
- **Sub-Epic**: Sub-Epic 3: Kernel Module Management CLI Surface
- **Status**: Completed

## Specification Details
Formally specified the CLI surface for Kernel Module Management (`aiosh mod` / `aiosh module`):

### 1. Command Syntax & Arguments
- `aiosh mod list [--store <path>] [--proc-modules <path>] [--json]`
- `aiosh mod show <name> [--proc-modules <path>] [--json]`
- `aiosh mod blacklist <module> [--store <path>] [--json]`
- `aiosh mod unblacklist <module> [--store <path>] [--json]`
- `aiosh mod options <module> <k=v...> [--store <path>] [--json]`
- `aiosh mod autoload <module> [--store <path>] [--json]`
- `aiosh mod unautoload <module> [--store <path>] [--json]`
- `aiosh mod preset list [--json]`
- `aiosh mod preset apply <preset_name> [--store <path>] [--json]`
- `aiosh mod export [--store <path>] [--modprobe <path>] [--autoload <path>] [--json]`

### 2. Exit Codes & JSON Schema
- **Exit Code 0**: Successful operation.
- **Exit Code 1**: Domain validation error, conflict error, or store I/O error (`INVALID_MODULE_NAME`, `CONFLICT_DETECTED`, `LOAD_STORE_FAILED`).
- **Exit Code 2**: CLI invocation error (missing required arguments, unrecognized options).
- **JSON Envelope**:
  - Success: `{"code": 0, "data": <T>, "error": null}`
  - Error: `{"code": 1, "data": null, "error": {"code": "<CODE>", "message": "<MSG>"}}`

### 3. Invariants (KC1..KC5)
- **KC1**: Command line validation (exit 2 on syntax errors).
- **KC2**: JSON envelope uniformity.
- **KC3**: Terminal output sanitization.
- **KC4**: Structured audit logging for state-mutating subcommands.
- **KC5**: Fail-closed error handling with specific error codes.
