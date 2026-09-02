# T-00565 — Evidence & Audit Trail / automated tests: Unit Test

## 1. Test Scope & Invariants
This task implements the comprehensive behavioral unit test suite `tools/test_check_evidence.py` for the Evidence Invariants & Verification Checker (`tools/check_evidence.py`).

In alignment with **T-00562** specification and repository-wide test style parity:
- Tests execute with standard library only (`unittest`/`tempfile`/`importlib`).
- Tests isolate the file system using a temporary sandbox (`Sandbox` with attribute rebinding).
- Evaluates happy path, negative path, boundary values, and mutation sensitivity (S01).

## 2. Test Cases & Coverage
- **E1 Directory Health**:
  - `U01`: Valid directory with markdown evidence files returns `True`.
  - `U02`: Non-existent evidence directory returns `False`.
  - `U03`: Empty evidence directory without `T-*.md` files returns `False`.
- **E2 Ledger Consistency**:
  - `U04`: Valid state with matching evidence files returns `True`.
  - `U05`: Missing `TASK_STATE.json` returns `False`.
  - `U06`: Corrupt/malformed JSON in `TASK_STATE.json` returns `False`.
  - `U07`: Missing evidence file for completed task in sample returns `False`.
  - `U08`: Boundary condition: empty `completed` list returns `True`.
- **E3 File Bounds & UTF-8 Integrity**:
  - `U09`: Valid non-empty UTF-8 files $\le 16\text{ MiB}$ return `True`.
  - `U10`: Empty file (0 bytes) returns `False`.
  - `U11`: Oversized file ($> 16\text{ MiB}$) returns `False`.
  - `U12`: Invalid non-UTF-8 bytes return `False`.
- **E4 Hash Consistency**:
  - `U13`: Deterministic SHA-256 computation returns `True`.
  - `U14`: Multiple evidence files verified deterministically.
- **S01 Mutation Sensitivity**:
  - `S01`: Simulates checker blindness (forcing `True` on empty file) and asserts detection by the test runner.

## 3. Verification Output
```text
Running Evidence Checker behavioral unit tests (T-00565)...
[+] U01 E1 directory-health: valid dir with files returns True
[+] U02 E1 directory-health: non-existent dir returns False
[+] U03 E1 directory-health: empty dir without evidence returns False
[+] U04 E2 ledger-consistency: valid state and matching files returns True
[+] U05 E2 ledger-consistency: missing state file returns False
[+] U06 E2 ledger-consistency: corrupt JSON returns False
[+] U07 E2 ledger-consistency: missing task file flagged as False
[+] U08 E2 ledger-consistency: empty completed list boundary returns True
[+] U09 E3 file-bounds: valid non-empty UTF-8 files return True
[+] U10 E3 file-bounds: empty file (0 bytes) returns False
[+] U11 E3 file-bounds: oversized file returns False
[+] U12 E3 file-bounds: invalid non-UTF-8 bytes return False
[+] U13 E4 hash-consistency: valid SHA-256 digest returns True
[+] U14 E4 hash-consistency: multiple evidence files verify cleanly
[+] S01 Sensitivity: checker blindness is detectable

Summary: 15/15 passed, 0 failed.
PASS: test_check_evidence_unit (15/15 checks green)
```
