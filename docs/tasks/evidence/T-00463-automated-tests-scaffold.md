# T-00463 — Documentation Index Control / automated tests: Scaffold

## 1. Scaffold Scope
This task creates the test runner harness skeleton `tools/test_doc_index_suites.py` covering criteria D1..D7, exports typed function signatures that fail loudly with `NotImplementedError` while `IS_IMPLEMENTED = False`, and verifies the scaffold interfaces using `tools/test_doc_index_scaffold.py`.

## 2. Scaffold Implementation
- `tools/test_doc_index_suites.py`:
  - `CRITERIA = ["D1", "D2", "D3", "D4", "D5", "D6", "D7"]`
  - `check_d1_manifest_model(repo_root: Path) -> tuple[bool, str]`
  - `check_d2_config_hierarchy(repo_root: Path) -> tuple[bool, str]`
  - `check_d3_title_and_link_extraction(repo_root: Path) -> tuple[bool, str]`
  - `check_d4_link_integrity_and_traversal(repo_root: Path) -> tuple[bool, str]`
  - `check_d5_cli_subcommands(repo_root: Path) -> tuple[bool, str]`
  - `check_d6_mcp_surface(repo_root: Path) -> tuple[bool, str]`
  - `check_d7_hardening_limits(repo_root: Path) -> tuple[bool, str]`
  - `run_all_criteria(repo_root: Path | None = None) -> bool`
  - `main() -> int`

- `tools/test_doc_index_scaffold.py`:
  - Validates interface existence, signature arity, and loud failures on all check functions.

## 3. Test Verification
```text
PASS: doc_index test scaffold — all interfaces present and fail loudly
```
