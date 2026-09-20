# Task Evidence: T-02026 - Capability Model / CLI surface: Integration (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02026`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Implement end-to-end integration and smoke tests for `aiosh capability` CLI commands.

---

## 2. Integration Test Implementation
- Created `code/aiosh-cli/tests/test_capability_cli_smoke.py`.
- Covers:
  1. `test_capability_help`: verifies `aiosh capability --help` displays all subcommands.
  2. `test_capability_unknown_subcommand`: verifies unknown commands return exit code 2.
  3. `test_capability_path_hygiene`: verifies path traversal and non-JSON extensions return exit code 2.
  4. `test_capability_lifecycle`:
     - Empty listing.
     - Root capability issuance with unauthorized issuer (returns 1).
     - Root capability issuance with authorized issuer `kernel` (returns 0).
     - Capability retrieval via `show`.
     - Monotonic attenuation via `attenuate`.
     - Fine-grained access verification via `check` (granted for read, denied for write).
     - Transitive cascade revocation via `revoke`.
     - Access verification post-revocation (denied).
     - Dead leaf pruning via `prune`.

---

## 3. Verification & Execution
- Python Integration Suite:
  - Command: `python code/aiosh-cli/tests/test_capability_cli_smoke.py`
  - Result: All tests passed.
