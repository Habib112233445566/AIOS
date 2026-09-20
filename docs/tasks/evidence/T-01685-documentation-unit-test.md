# T-01685: Kernel Module Management Documentation Unit Tests

## Sub-Epic
Kernel Module Management / Documentation (T-01685)

## Objective
Author and execute in-tree Rust unit tests for the Kernel Module Management Documentation subsystem, validating invariants KD1 through KD6.

## Test Suite Implementation (`test_kernel_module_doc.rs`)

1. **`test_kd1_index_initialization_and_canonical_topics` (KD1)**:
   - Verifies `KernelModuleDocIndex::new()` initializes with at least 7 canonical topics:
     - `modprobe-directives`
     - `cis-benchmark-hardening`
     - `lifecycle-workflows`
     - `observability-and-procfs`
     - `security-policy-and-pep`
     - `container-isolation`
     - `wireless-pentest`

2. **`test_kd2_topic_lookup_and_case_insensitivity` (KD2)**:
   - Confirms case-insensitive retrieval (`modprobe-directives` vs `MODPROBE-DIRECTIVES`).
   - Asserts safe `None` return for unknown topic IDs.

3. **`test_kd3_search_scoring_and_ranking` (KD3)**:
   - Validates multi-tier search ranking (exact ID match $\ge 100$, tag match $\ge 50$, content match $\ge 10$).
   - Validates empty query handling.

4. **`test_kd4_category_filtering`**:
   - Asserts category filtering for `DocCategory::Security` and `DocCategory::Baseline`.

5. **`test_kd5_markdown_rendering_quality` (KD4)**:
   - Verifies `format_topic_markdown` produces formatted Markdown with headers, metadata line, code blocks, and references.

6. **`test_kd6_serialization_and_json_export` (KD5)**:
   - Tests lossless JSON serialization/deserialization for topics and search result arrays.

## Test Execution Results
```
running 6 tests
test test_kd1_index_initialization_and_canonical_topics ... ok
test test_kd2_topic_lookup_and_case_insensitivity ... ok
test test_kd4_category_filtering ... ok
test test_kd3_search_scoring_and_ranking ... ok
test test_kd5_markdown_rendering_quality ... ok
test test_kd6_serialization_and_json_export ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```
