# T-00534 — Evidence & Audit Trail / CLI surface: Implementation

## 1. Implementation Scope
This task implements the complete `aiosh evidence` CLI command suite in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. CLI Implementation Details
- `aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]`:
  - Validates manifests against on-disk files and outputs structured success/failure summaries.
- `aiosh evidence hash <path> [--json]`:
  - Calculates deterministic SHA-256 hex checksums for specified files.
- `aiosh evidence scan [--repo <path>] [--task <id>] [--json]`:
  - Discovers and computes SHA-256 checksums for all task evidence files under `docs/tasks/evidence/`.
- All operations log structured events to the SQLite WAL audit ring.

## 3. Test Verification
```text
PASS: aiosh evidence hash prose
PASS: aiosh evidence hash --json
PASS: aiosh evidence verify --json
PASS: aiosh evidence scan --json
All evidence CLI smoke tests passed successfully!
```
