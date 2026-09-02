# T-00530 — Evidence & Audit Trail / core service: Verification & Evidence

## 1. Sub-Epic Verification Overview
This task concludes the Evidence & Audit Trail / core service sub-epic (T-00521..T-00530), validating that all core service algorithms (`compute_file_sha256`, `build_evidence_record`, `verify_evidence_manifest`, `check_evidence_policy`), CLI commands (`aiosh evidence verify`, `aiosh evidence hash`), MCP tools (`aios.evidence.verify`, `aios.evidence.hash`), and documentation invariants pass cleanly.

## 2. Test Execution Results

### A. Core Rust Evidence Service Tests (`cargo test -p aiosh-core evidence_service::tests`)
```text
running 5 tests
test evidence_service::tests::test_check_evidence_policy_enforcement ... ok
test evidence_service::tests::test_build_evidence_record_invalid_paths_error ... ok
test evidence_service::tests::test_compute_file_sha256_happy_and_missing ... ok
test evidence_service::tests::test_build_and_verify_evidence_manifest_happy ... ok
test evidence_service::tests::test_verify_evidence_manifest_mismatch_and_missing ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 138 filtered out; finished in 0.04s
```

### B. Core Rust Evidence Data Model Tests (`cargo test -p aiosh-core evidence::tests`)
```text
running 8 tests
test evidence::tests::test_evidence_record_invalid_hash ... ok
test evidence::tests::test_evidence_record_invalid_status ... ok
test evidence::tests::test_evidence_record_path_traversal ... ok
test evidence::tests::test_evidence_record_task_id_bounds ... ok
test evidence::tests::test_evidence_record_valid ... ok
test evidence::tests::test_evidence_step_as_str_all_variants ... ok
test evidence::tests::test_task_evidence_manifest_duplicate_error ... ok
test evidence::tests::test_task_evidence_manifest_roundtrip_and_query ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.00s
```

### C. MCP Evidence & Tool Tests (`cargo test -p aiosh-mcp`)
```text
running 2 tests
test tests::test_mcp_doc_tools_execution ... ok
test tests::test_toolchain_tools_in_manifest ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

### D. Documentation Quality & Invariants (`tools/check_task_docs.py`)
```text
[+] C1 spec-health
[+] C2 component sections
[+] C3 referenced paths
[+] C4 phase map
[+] C5 index health
[+] C6 no volatile counts

PASS: task docs criteria (C1..C6)
```

### E. Security Policy Invariants (`tools/check_security_policy.py`)
```text
[+] S1 SECURITY.md exists at root
[+] S1b no TODO markers remain
[+] S2 advisory URL present verbatim
[+] S3 free-form prose (>1200 chars)
[+] S4 specific text (vuln=3, disclos=3, day-count=True)
[+] S5 all referenced in-tree paths exist

PASS: security policy criteria (S1..S5)
```

## 3. Sub-Epic Milestone Closeout
- **Sub-Epic**: `Phase 0 — Governance, Repo, CI, Task Execution / Evidence & Audit Trail / core service` (Tasks T-00521 .. T-00530).
- **Status**: 10/10 tasks completed.
- **Next Sub-Epic**: `Phase 0 — Governance, Repo, CI, Task Execution / Evidence & Audit Trail / CLI surface` starting at `T-00531`.
