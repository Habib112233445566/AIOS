# T-02325: Audit Chain Extensions CLI Surface Unit Test

## Overview
This task delivers automated unit tests verifying the CLI surface for Audit Chain Extensions in `code/aiosh-rust/aiosh-core/tests/test_audit_chain_cli.rs`.

## Test Execution Results

```text
cargo test --test test_audit_chain_cli
   Compiling aiosh-core v0.1.0 (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\aiosh-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.97s
     Running tests\test_audit_chain_cli.rs (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\deps\test_audit_chain_cli-714f2d583a5e1584.exe)

running 2 tests
test test_cli_audit_query_and_inspect ... ok
test test_cli_audit_ancestry_and_sign_verify ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
```

## Coverage Summary
1. `test_cli_audit_query_and_inspect`:
   - Validates `aiosh audit query --session <session-id>` returns matching records within the standard JSON envelope.
   - Validates `aiosh audit inspect <hash>` outputs the complete extended record.
2. `test_cli_audit_ancestry_and_sign_verify`:
   - Validates `aiosh audit ancestry <hash> --depth 5` traces parent DAG dependencies accurately.
   - Validates `aiosh audit sign-verify <hash>` verifies cryptographic digital signatures.
   - Asserts negative case: missing required arguments trigger exit code `2` (usage refusal).
