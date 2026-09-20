# T-02121: CLI Surface Research — PEP Decision Engine

## Overview
- **Task ID**: `T-02121`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## 1. Existing Codebase Analysis
- **Binary Architecture**: `code/aiosh-rust/aiosh-cli/src/main.rs` is the primary executable for `aiosh`. All subcommands (`task`, `capability`, `update`, `service`, `mod`, `hw`, `net`) dispatch from `fn main()`.
- **Context & Audit**: Every subcommand initializes `let mut ctx = open_context();` and emits an audit record via `classify_and_emit` or `emit`.
- **Exit Code Convention**:
  - `0`: Success (for evaluation: `Permit`).
  - `1`: Operational failure or access denied (for evaluation: `Deny`).
  - `2`: Syntax, argument validation, or path hygiene error.
- **Output Mode**: Plain text by default, or structured JSON envelope when `--json` is supplied (`{ "code": i32, "data": Value, "error": Value }`).

## 2. Authoritative Sources & Standards
1. **NIST SP 800-162**: Attribute Based Access Control (ABAC) Definition and Considerations. Defines PDP/PEP interfaces, request attributes (subject, resource, action, environment), and rule evaluation semantics.
2. **Open Policy Agent (OPA) CLI**: CLI design patterns for evaluating policies against input data (`opa eval`), showing status, and managing rule sets.
3. **AIOS ADR-0035 §D-2, §D-4**:
   - Thin CLI architecture: CLI commands emit exactly one audit row.
   - Complete mediation: all actions are mediated and auditable.

## 3. Fact vs. Assumption Matrix
| Item | Classification | Description |
|---|---|---|
| `PepDecisionService` API | Fact | Implemented in `aiosh_core::pep_decision_service` with rule CRUD and evaluation methods. |
| Path Traversal Validation | Fact | Mandatory in all AIOS CLI commands; must reject `..` and non-JSON paths. |
| Default Policy Store | Fact | Resides at `$AIOSH_HOME/pep_policies.json` (fallback: `~/.aios/pep_policies.json`). |
| Operator Subcommands | Decision | Selected subcommands: `evaluate`, `rule-add`, `rule-list`, `rule-remove`, `status`. |
| Evaluation Exit Codes | Decision | Exit code 0 for `Permit`, exit code 1 for `Deny`, exit code 2 for invalid input. |

## 4. Unknowns & Resolved Decisions
- **Rule Priorities**: Default priority is `0` when omitted in `rule-add`.
- **Terminal Sanitization**: All error strings and outputs are passed through `sanitize_terminal` to prevent ANSI escape injection attacks.
- **Rule Identification**: Rule IDs must be alphanumeric with underscores/hyphens, max 128 characters.
