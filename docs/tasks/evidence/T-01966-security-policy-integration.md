# Task Evidence: T-01966 (System Update / security policy: Integration)

## Summary
Integrated the System Update Security Policy Subsystem across Rust core and Python/MCP client harnesses:
1. `code/aiosh-rust/aiosh-core/src/system_update_policy.rs`: Exported in `lib.rs` and validated with 9/9 unit tests passing in `test_system_update_policy.rs`.
2. `code/aiosh-mcp/tests/test_system_update_policy_smoke.py`: Authoritative Python smoke and integration suite validating invariants UPOL1..UPOL6 (channel enforcement, signature check, anti-rollback downgrade prevention, partition target governance, quota caps, and revocation denylist).
3. Cross-substrate parity: Verified that policy schemas, violation records, and evaluation reports serialize to canonical JSON identical across Rust and Python.

## Verification
- `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update_policy`: **PASS** (9/9 passed in 0.02s)
- `python code/aiosh-mcp/tests/test_system_update_policy_smoke.py`: **PASS** (7/7 checks pass)
