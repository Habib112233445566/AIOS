# T-01025 — Distro Selection & Justification / CLI Surface: Unit Test

## 1. Test Suite Overview
Created dedicated smoke and unit test suite `code/aiosh-cli/tests/test_distro_cli_smoke.py`:
- `test_distro_list_prose`: Validates human-readable table output.
- `test_distro_list_json`: Asserts JSON array response and presence of required profile IDs.
- `test_distro_show_prose`: Validates detailed field rendering for `debian-12-minimal-x86_64`.
- `test_distro_show_json`: Verifies JSON deserialization, family, and recommended status.
- `test_distro_evaluate_all`: Verifies array scoring and ranking order.
- `test_distro_evaluate_single`: Verifies targeted evaluation for `alpine-319-container-x86_64`.
- `test_distro_recommend`: Verifies reference profile retrieval.
- `test_distro_help`: Verifies `--help` flag returns 0 and displays syntax.
- `test_distro_missing_id`: Negative case asserting missing argument returns exit code `2`.
- `test_distro_not_found`: Negative case asserting nonexistent ID returns exit code `1`.

## 2. Standalone Test Output
```
PASS: aiosh distro list prose
PASS: aiosh distro list --json
PASS: aiosh distro show prose
PASS: aiosh distro show --json
PASS: aiosh distro evaluate --json
PASS: aiosh distro evaluate <id> --json
PASS: aiosh distro recommend --json
PASS: aiosh distro --help
PASS: aiosh distro show missing id returns 2
PASS: aiosh distro show nonexistent returns 1

ALL DISTRO CLI SMOKE TESTS PASSED!
```
