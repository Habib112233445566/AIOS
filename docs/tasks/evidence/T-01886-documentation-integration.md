# Task Evidence: T-01886 - Network Bootstrap / documentation: Integration

## 1. Overview
- **Task ID**: `T-01886`
- **Sub-Epic**: 9 (Network Bootstrap Documentation Subsystem)
- **Goal**: Implement and verify integration test suite for Network Bootstrap Documentation Subsystem in `code/aiosh-cli/tests/test_network_doc_smoke.py`.

---

## 2. Integration Verification Scope
- Verified end-to-end integration and invariants across surfaces:
  - `NDOC1`: Canonical offline documentation topic catalog pre-population (architecture, discovery, security policy, observability, configuration, troubleshooting).
  - `NDOC2`: Loose category alias parsing (`arch`, `probe`, `sec`, `obs`, `cfg`, `triage`) and case-insensitive topic filtering.
  - `NDOC3`: Multi-field ranked search scoring engine prioritizing ID (+100), Title (+50), Tag (+25), Summary (+20), and Section (+5) with query bounds.
  - `NDOC4`: Markdown topic rendering with metadata headers, section bodies, code blocks, RFC references, and tags.
  - `NDOC5`: Dynamic state Markdown generation and hierarchical ASCII topology rendering.
  - `NDOC6`: JSON schema serialization parity, atomic persistence via temporary file swap, path hygiene, and 1 MB file cap enforcement.

---

## 3. Test Execution Verification
Command: `python code/aiosh-cli/tests/test_network_doc_smoke.py`

Output:
```text
Running Network Bootstrap Documentation Smoke Tests (T-01886)...
PASS: test_ndoc1_canonical_repository
PASS: test_ndoc2_loose_category_matching
PASS: test_ndoc3_search_ranking
PASS: test_ndoc4_markdown_topic_rendering
PASS: test_ndoc5_dynamic_state_and_ascii_topology
PASS: test_ndoc6_json_parity_and_file_bounds
ALL NETWORK DOCUMENTATION SMOKE TESTS PASSED.
```
Status: Verified and Passed (6/6 smoke tests passed, 0 failures).
