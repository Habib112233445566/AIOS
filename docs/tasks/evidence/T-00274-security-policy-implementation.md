# T-00274 — Security Policy: Implementation

## Implementation Details
We implemented the security policy for Release Packaging & Backup natively in Rust, integrating it with the existing PEP (Policy Enforcement Point) logic.
- **PEP Updates**: Added `aios.backup.*` and `aios.release.*` prefixes to the `is_irreversible` matcher in `aiosh-core/src/pep.rs`. This guarantees that these tools can never bypass the dispatcher gate without explicit, cryptographically verifiable grants.
- **Handler Verification**: Replaced the scaffold in `aiosh-core/src/release.rs` with `check_release_policy`, which invokes `crate::pep::is_irreversible` and enforces that a grant is provided.
- **Audit Logging**: As seen in `release.rs`, the generation and backup functions already emit `ReleaseGenerated`/`BackupCreated` audit rows via `ctx.ring.write(AuditRowInput { ... })`, satisfying the ADR-0035 audit invariants natively.

## Validation Results
We ran the full test suite via `cargo test`, resulting in 76 passed tests. The `test_check_release_policy_enforcement` explicitly validates the fail-closed behavior for missing grants.

```text
running 76 tests
...
test release::security_tests::test_check_release_policy_enforcement ... ok
...
test result: ok. 76 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.84s
```

## Conclusion
The automated enforcement of the Release Packaging & Backup security policy is complete and leverages the shared PEP capabilities cleanly.
