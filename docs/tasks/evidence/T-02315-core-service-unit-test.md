# T-02315: Audit Chain Extensions Core Service Unit Test

## Overview
This task adds comprehensive automated unit tests for `AuditChainService` in `code/aiosh-rust/aiosh-core/tests/test_audit_chain_service.rs`.

## Test Execution Results

```text
cargo test --test test_audit_chain_service
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 10s
     Running tests\test_audit_chain_service.rs (C:\Users\OBSESSION\Desktop\AIOS_MERGED\code\aiosh-rust\target\debug\deps\test_audit_chain_service-47cb2a7710fb54bb.exe)

running 5 tests
test test_service_query_filtering ... ok
test test_service_record_and_get_by_hash ... ok
test test_service_record_bounds_enforcement ... ok
test test_service_signature_verification ... ok
test test_service_trace_ancestry_dag ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

## Coverage Details
1. `test_service_record_and_get_by_hash`: Tests recording an extended audit row and exact hash lookup with full attribute preservation.
2. `test_service_query_filtering`: Validates multi-attribute query indexing across `session_id`, `trace_id`, and `tool`.
3. `test_service_trace_ancestry_dag`: Verifies multi-hop DAG traversal across 3 generations (root -> child -> grandchild) with relationship metadata and depth markers.
4. `test_service_signature_verification`: Verifies asymmetric cryptographic signatures.
5. `test_service_record_bounds_enforcement`: Asserts that boundary and format violations are rejected before database commit.
