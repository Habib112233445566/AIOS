# T-00283 — Release Packaging & Backup: Observability Scaffold

## Scaffold Implementation
We created the skeleton for the observability enhancements by extending `aiosh-core/src/release.rs`. 
Specifically, we added the strongly-typed function signature:

```rust
pub fn run_external_packager(_cmd: &str, _args: &[&str]) -> Result<(), String>
```

This function will eventually replace the simple raw OS command spawning by introducing a unified stderr capture logic.

As required by the scaffolding phase, the body currently fails loudly with `unimplemented!()`:
```rust
unimplemented!("Scaffolded: Capture stderr for observability");
```

## Validation
- The project successfully compiles with `cargo check` and `cargo test`.
- We added a test `test_run_external_packager_scaffold` which is decorated with `#[should_panic(expected = "Scaffolded: Capture stderr for observability")]`. This test natively verifies the fail-loud invariant is maintained.

```text
test release::observability_tests::test_run_external_packager_scaffold - should panic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 76 filtered out; finished in 0.00s
```

The scaffolding is successfully wired and ready for the implementation phase.
