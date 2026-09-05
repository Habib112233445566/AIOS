# T-01221: Package Management - CLI Surface: Research

## Metadata
- **Task ID:** `T-01221`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Research
- **Status:** Complete

## 1. Objectives & Scope
Research the operator-facing command-line interface (`aiosh package`) for querying, inspecting, validating, planning, and executing package transactions on AIOS. The CLI surface must satisfy:
1. **Interactive Usability**: Clear, tabular terminal output for human operators with color-coded or aligned metadata.
2. **Machine Consumption**: Deterministic, structured JSON envelopes when invoked with `--json` for automation and agent parsing.
3. **Auditability (ADR-0035)**: Synchronous audit emission via `classify_and_emit` on every CLI invocation (both success and failure).
4. **Fail-Closed Defensive Sizing**: Bounded input reads (1 MiB ceiling on payloads and files) and rejection of invalid arguments with POSIX exit codes.

## 2. Prior Art & Authoritative Sources
- **Debian Policy Manual (§5.6.7 - §5.6.10)**: Package relationships (`Depends`, `Pre-Depends`, `Recommends`, `Suggests`, `Conflicts`) and Debian naming grammar (`^[a-z0-9][a-z0-9+.-]*$`).
- **Debian `apt(8)` / `dpkg(1)` CLI Conventions**: Standard subcommands (`list`, `show`, `search`, `install`, `remove`), dry-run flags (`-s`, `--simulate`, `--dry-run`), and human-oriented summary lines.
- **Alpine Linux `apk(8)` CLI Manual**: Compact command syntax, world dependency tracking, and package state representation.
- **POSIX.1-2017 Utility Conventions (IEEE Std 1003.1)**: Guidelines 3 through 10 governing standard flag syntax, arguments, and exit code semantics.

## 3. Fact vs. Assumption Separation

### Established Facts
- **Fact 1 (Data Model & Core Service)**: `aiosh-core::package` and `aiosh-core::package_service` provide `PackageSpec`, `PackageStore`, `plan_transaction`, and `execute_transaction` with verified invariants `PM1..PM5` and `CS1..CS5`.
- **Fact 2 (Existing Subcommands)**: `aiosh package` already implements `validate` (name and spec), `list` (with format, state, pattern, limit filters), `show` (by name), and `plan` (via `--actions`).
- **Fact 3 (Audit Emission)**: `classify_and_emit` is integrated in `aiosh-cli` and records every execution to the SQLite WAL ring (`audit.db`).
- **Fact 4 (Memory Protection)**: Payload reads for spec files and actions are capped at 1 MiB (`1,048,576` bytes).

### Engineering Assumptions
- **Assumption 1**: Operators will benefit from convenience verbs (`install`, `remove`, `search`) in addition to low-level transaction planning (`plan`).
- **Assumption 2**: Interactive installations should support a `--yes` / `-y` flag to bypass interactive confirmation prompts, while dry-run mode (`--dry-run`) provides safe non-destructive previews.
- **Assumption 3**: Store location can be overridden via `--store <path>` or the `AIOS_PACKAGE_STORE` environment variable, falling back to a canonical local default.

## 4. Proposed CLI Command Grammar
```text
aiosh package <subcommand> [options]

Subcommands:
  validate   --name <name> | --spec <file_or_json> [--json]
  list       [--format <deb|apk|flatpak|tarball>] [--state <state>] [--pattern <str>] [--limit <n>] [--json] [--store <path>]
  show       <name> [--json] [--store <path>]
  search     <pattern> [--json] [--store <path>]
  plan       --actions <file_or_json> [--dry-run] [--json] [--store <path>]
  apply      --plan <file_or_json> [--yes] [--json] [--store <path>]
```

## 5. Exit Code Contract
- `0`: Success (package found, list generated, plan computed, transaction applied).
- `1`: Operational error (package not found, store file unreadable, persistence failed).
- `2`: Syntax or invocation error (missing required arguments, invalid JSON, payload > 1 MiB, invalid format/state value).

## 6. Decisions & Unknowns for Specification Phase (T-01222)
- **Decision 1**: Formalize `search` as an alias for `list --pattern <pattern>` to align with operator intuition (`apt search` / `apk search`).
- **Decision 2**: Keep `plan` and `apply` decoupled so agents and operators can inspect estimated disk delta before committing changes to the persistent store.
- **Decision 3**: Standardize exit code envelopes for all new verbs to guarantee deterministic JSON parsing across the entire CLI surface.
