# T-00692 — Repository Health / documentation: Specification

## Specification & Documentation Invariants

### 1. Document Structure & Invariants
- `docs/README.md` MUST maintain the `## Repository Health (T-00611..T-00710)` section.
- Backticked file and code paths MUST resolve to valid in-tree files per `tools/check_task_docs.py` (C3).
- Must avoid volatile pass/fail count snapshots in documentation prose (C6).
- All referenced task evidence paths must exist in `docs/tasks/evidence/`.

### 2. Formatter Contract (`format_repo_health_summary`)
- Human-readable rendering for terminal operator inspection:
  - Header: Repository root and overall status banner (`[PASS]`, `[WARN]`, `[FAIL]`).
  - Table / List: Check ID, name, status, message, and elapsed duration (`<N>ms`).
  - Summary footer: Total checks, counts by status, and overall verdict.
- Structured `--json` serialization matching `RepoHealthReport` serialization exactly.

### 3. Error Handling & Limitations
- If a check produces warnings (e.g. uncommitted files or oversized assets), the formatter must list the first 50 items and state if details were truncated.
- Missing configuration files fallback to defaults and note default resolution in stdout.

### 4. Out of Scope
- Interactive TUI or GUI dashboard rendering (Phase 4).
- Automatic staging/committing of untracked files or automatic file deletion.
