# T-00434 — Documentation Index Control / CLI surface: Implementation

## 1. Implementation Scope
This task implements the CLI surface commands for Documentation Index Control in `code/aiosh-rust/aiosh-cli/src/main.rs`.

## 2. Command Implementations
- **`aiosh doc show [--repo <path>] [--json]`**:
  - Compiles documentation catalog into `DocIndexManifest`.
  - Supports standard human-readable prose table or `--json` output envelopes.
  - Emits `doc.show` audit row.
- **`aiosh doc check [--repo <path>] [--json]`**:
  - Validates in-tree markdown links across documents and flags missing target files or root escapes.
  - Returns `0` on clean validation, `1` if broken links are discovered.
  - Emits `doc.check` audit row.
- **`aiosh doc search <query> [--repo <path>] [--json]`**:
  - Filters indexed documents by substring query matching across path, title, and section.
  - Emits `doc.search` audit row.

## 3. Test Verification
```text
running 1 test
test task_cli_tests::test_cmd_doc_show_check_and_search ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.17s
```
