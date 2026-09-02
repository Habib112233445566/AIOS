# T-00432 — Documentation Index Control / CLI surface: Specification

## 1. Specification Overview
This document specifies the command-line interface contract, flags, exit codes, and output envelopes for the `aiosh doc` CLI surface in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Command Synopsis & Subcommands

### A. `aiosh doc show`
- **Usage**: `aiosh doc show [--json]`
- **Description**: Displays the active documentation catalog grouped by section.
- **Output Formats**:
  - *Standard Prose*:
    ```text
    AIOS Documentation Index (v1.0.0):
      [Documentation]
        - Main Documentation (docs/README.md) [T-00001..T-00500]
        - Task Ledger Invariants (docs/SPEC-TASK-LEDGER.md)
      [Governance]
        - Goals & Sequential Laws (docs/tasks/GOALS.md)
    ```
  - *JSON Format (`--json`)*: Serializes `DocIndexManifest` as formatted JSON.
- **Exit Code**: `0` on success.

### B. `aiosh doc check`
- **Usage**: `aiosh doc check [--repo <path>] [--json]`
- **Description**: Validates that all internal markdown links resolve to existing files within repository bounds.
- **Output Formats**:
  - *Success*:
    ```text
    [+] Documentation link verification passed (14 links checked)
    ```
  - *Failure*:
    ```text
    [-] Broken links detected (14 links checked, 2 broken):
        - docs/README.md -> docs/missing.md (Target file does not exist on disk)
    ```
- **Exit Codes**:
  - `0`: All links valid.
  - `1`: One or more broken/escaping links detected.
  - `2`: Document read or argument error.

### C. `aiosh doc search`
- **Usage**: `aiosh doc search <query> [--json]`
- **Description**: Case-insensitive substring search across indexed document paths, titles, and sections.
- **Exit Code**: `0` on success, `2` if `<query>` is omitted.

## 3. Error Handling & Exit Codes
- `0`: Normal completion / successful validation.
- `1`: Conformance / link validation failure.
- `2`: Invalid argument or unknown subcommand.

## 4. PEP & Audit Invariants
- `aiosh doc` commands are strictly read-only diagnostics and do not mutate system state.
