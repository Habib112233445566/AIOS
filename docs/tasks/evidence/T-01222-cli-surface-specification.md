# T-01222: Package Management - CLI Surface: Specification

## Metadata
- **Task ID:** `T-01222`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Package Management CLI Surface Specification
- **Status:** Complete

## 1. CLI Dispatch Specification (`aiosh package`)

### 1.1 Command Interface & Entry Point
```rust
fn cmd_package(args: &[String]) -> i32
```
Dispatched in `aiosh-cli/src/main.rs`:
```rust
Some("package") => cmd_package(&args[1..]),
```

### 1.2 Subcommands & Contract

#### 1. `validate`
- **Syntax**: `aiosh package validate (--name <name> | --spec <file_or_json>) [--json]`
- **Inputs**: Package name string or serialized `PackageSpec`.
- **Validation**: Enforces 1 MiB payload ceiling. Validates PM1 naming syntax and PM1..PM5 spec invariants.
- **Output**:
  - Text: `VALID: ...` or `INVALID: ...`.
  - JSON: Standard result envelope `{"code": 0|2, "data": ..., "error": ...}`.
- **Exit Codes**: `0` on valid, `2` on validation or usage error.
- **Persistence**: Read-only; zero disk side effects.

#### 2. `list`
- **Syntax**: `aiosh package list [--format <deb|apk|flatpak|tarball>] [--state <state>] [--pattern <str>] [--limit <n>] [--json] [--store <path>]`
- **Inputs**: Filter flags and optional custom store path.
- **Output**:
  - Text: Formatted table with `NAME`, `VERSION`, `FORMAT`, `STATE`, `INSTALLED SIZE`.
  - JSON: Array of serialized `PackageSpec` objects.
- **Exit Codes**: `0` on success, `1` on store loading error, `2` on invalid argument.
- **Persistence**: Read-only.

#### 3. `show <name>`
- **Syntax**: `aiosh package show <name> [--json] [--store <path>]`
- **Inputs**: Package name identifier and optional store path.
- **Output**:
  - Text: Multi-line attribute display (version, format, state, size, description, dependencies).
  - JSON: Serialized `PackageSpec`.
- **Exit Codes**: `0` on found, `1` on store load failure, `2` if missing name or package not found.
- **Persistence**: Read-only.

#### 4. `search <pattern>` (AIOS-specific convenience shortcut)
- **Syntax**: `aiosh package search <pattern> [--json] [--store <path>]`
- **Inputs**: Search substring pattern.
- **Behavior**: Equivalent to `aiosh package list --pattern <pattern>`.
- **Exit Codes**: `0` on success, `2` if pattern argument missing.
- **Persistence**: Read-only.

#### 5. `plan`
- **Syntax**: `aiosh package plan --actions <file_or_json> [--dry-run] [--json] [--store <path>]`
- **Inputs**: JSON array of `PackageAction` (`install`, `remove`, `upgrade`, `purge`) or path to file.
- **Validation**: 1 MiB input size ceiling; `1..=256` actions; dependency closure (`CS3`); delta arithmetic (`CS4`).
- **Output**:
  - Text: Planned transaction ID, action count, delta bytes, and action preview.
  - JSON: Serialized `PackageTransaction`.
- **Exit Codes**: `0` on success, `1` on store load failure, `2` on invalid input or planning invariant violation.
- **Persistence**: Pure planning; store unmodified.

#### 6. `apply` (AIOS-specific transaction execution)
- **Syntax**: `aiosh package apply (--actions <file_or_json> | --plan <file_or_json>) [--dry-run] [--yes] [--json] [--store <path>]`
- **Inputs**: Transaction actions or pre-computed transaction plan.
- **Validation**: Validates transaction invariants, computes execution report.
- **Output**:
  - Text: Summary of installed, removed, upgraded packages and net disk delta.
  - JSON: Serialized `TransactionReport`.
- **Exit Codes**: `0` on success, `1` on persistence error, `2` on planning or execution failure.
- **Persistence Effects**: Modifies package store in-place and persists to disk via atomic temporary file write with RAII cleanup on error.

### 1.3 Audit Trail Guarantee (ADR-0035)
Every subcommand execution unconditionally invokes:
```rust
classify_and_emit(
    &mut ctx,
    "package",
    subcommand_name,
    json_params,
    outcome,
    target_name_or_id,
    detail,
    "operator",
    None,
);
```
Emitting a SHA-256 hash-chained audit row to `audit.db`.
