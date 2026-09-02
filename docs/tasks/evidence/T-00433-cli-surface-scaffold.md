# T-00433 — Documentation Index Control / CLI surface: Scaffold

## 1. Scaffold Scope
This task routes `aiosh doc` in `code/aiosh-rust/aiosh-cli/src/main.rs` to a skeleton `cmd_doc` dispatcher and verifies it compiles cleanly with a `#[should_panic]` test stub.

## 2. Scaffold Implementation
- In `main()`: Added routing `Some("doc") => cmd_doc(&args[1..])`.
- In `cmd_doc`: Added stub `unimplemented!("T-00433: aiosh doc scaffold")`.
- In `task_cli_tests`: Added `test_cmd_doc_scaffold`.

## 3. Test Verification
```text
running 1 test
test task_cli_tests::test_cmd_doc_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.01s
```
