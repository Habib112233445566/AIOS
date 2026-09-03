# T-01023 — Distro Selection & Justification / CLI Surface: Scaffold

## 1. Scaffolding Structure
- Defined `cmd_distro` CLI dispatch handler in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- Created cross-substrate smoke runner `code/aiosh-cli/tests/test_distro_cli_smoke.py` defining test suites for:
  - `test_distro_list_prose`
  - `test_distro_list_json`
  - `test_distro_show_prose`
  - `test_distro_show_json`
  - `test_distro_evaluate_all`
  - `test_distro_evaluate_single`
  - `test_distro_recommend`
  - `test_distro_help`
  - `test_distro_missing_id`
  - `test_distro_not_found`

## 2. Compilation Verification
- `cargo build --manifest-path code/aiosh-rust/Cargo.toml --bin aiosh`: Build verified clean with 0 errors.
- `python code/aiosh-cli/tests/test_distro_cli_smoke.py`: Initial test hooks callable and resolving binary correctly.
