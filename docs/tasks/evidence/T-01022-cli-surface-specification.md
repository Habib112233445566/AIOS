# T-01022 — Distro Selection & Justification / CLI Surface: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / CLI Surface

## 1. CLI Dispatch Specification (`aiosh distro`)

### 1.1 Command Interface & Syntax
```rust
fn cmd_distro(args: &[String]) -> i32
```
Registered under root `main()` dispatcher:
```rust
Some("distro") => cmd_distro(&args[1..]),
```

### 1.2 Subcommands & Arguments
1. **`list`**
   - **Syntax**: `aiosh distro list [--json] [--store <path>]`
   - **Behavior**: Reads active `DistroStore` and lists all profiles sorted by ID.
   - **Output**: Formatted ASCII table with columns `ID`, `FAMILY`, `ARCH`, `RECOMMENDED`, `NAME`. When `--json` is supplied, emits JSON array of `DistroProfile`.
   - **Exit Code**: `0` on success.

2. **`show <id>`**
   - **Syntax**: `aiosh distro show <id> [--json] [--store <path>]`
   - **Validation**: `<id>` must not start with `--`. If missing, returns code `2` with usage instructions.
   - **Behavior**: Finds profile by `<id>` in `DistroStore`. If found, displays full fields (name, id, family, arch, init_system, c_lib, min_kernel_version, default_packages, justification).
   - **Exit Code**: `0` on found, `1` if not found with error `Distro profile '<id>' not found`.

3. **`evaluate [<id>]`**
   - **Syntax**: `aiosh distro evaluate [<id>] [--json] [--store <path>]`
   - **Behavior**: If `<id>` provided, computes and displays `DistroEvaluation` for target profile. If omitted, computes evaluations for all profiles and sorts by `overall_score` descending.
   - **Exit Code**: `0` on success, `1` if requested `<id>` does not exist.

4. **`recommend`**
   - **Syntax**: `aiosh distro recommend [--json] [--store <path>]`
   - **Behavior**: Retrieves profile with `recommended == true`. Displays summary or JSON representation.
   - **Exit Code**: `0` on success, `1` if no recommended profile configured.

5. **`--help`, `-h`**
   - **Syntax**: `aiosh distro --help`
   - **Behavior**: Prints usage string and returns code `0`.

### 1.3 Audit Logging Invariant
Every subcommand execution invokes:
```rust
classify_and_emit(
    &mut ctx,
    "distro",
    subcommand_name,
    json_params,
    outcome,
    target_id,
    detail,
    "operator",
    None,
);
```
Guaranteeing a tamper-evident audit record in `AuditRing`.
