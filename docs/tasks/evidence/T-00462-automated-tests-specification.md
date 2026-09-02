# T-00462 — Documentation Index Control / automated tests: Specification

## 1. Specification Overview
This document specifies the test harness architecture, test assertion criteria (D1..D7), exit codes, and execution model for `tools/test_doc_index_suites.py`.

## 2. Test Runner Contract

### Execution Signature:
```bash
python tools/test_doc_index_suites.py
```

### Exit Codes:
- `0`: All criteria D1..D7 satisfied.
- `1`: One or more assertion failures.

### Standard Output Format:
```text
[+] D1 manifest model & query helpers
[+] D2 configuration hierarchy & limits
[+] D3 title parsing & link extraction
[+] D4 link integrity & traversal detection
[+] D5 CLI subcommand execution & json mode
[+] D6 MCP tool execution & protocol schemas
[+] D7 hardening limits & negative error bounds

PASS: doc_index test criteria (D1..D7)
```

## 3. Criteria Specifications

1. **D1 (Manifest Data Model)**:
   - Verifies `DocIndexManifest` JSON serialization, duplicate path detection, and entry query filters.
2. **D2 (Configuration Layer)**:
   - Validates `DocIndexConfig` defaults, 64 KiB size bounds, and env var overrides.
3. **D3 (Title & Link Extraction)**:
   - Validates parsing of `# Title` and inline relative Markdown links `[text](target.md)`.
4. **D4 (Link Integrity & Traversal Detection)**:
   - Asserts detection of missing target files and paths attempting out-of-bounds `../` directory escapes.
5. **D5 (CLI Interface Parity)**:
   - Tests `aiosh doc show`, `aiosh doc check`, and `aiosh doc search` in prose and `--json` modes.
6. **D6 (MCP Surface Conformance)**:
   - Tests `aios.doc.index.get`, `aios.doc.check`, and `aios.doc.search` over JSON-RPC stdio protocol.
7. **D7 (Hardening Limits & Negative Tests)**:
   - Verifies rejection of oversized configs, invalid extensions, and malformed inputs with clean error envelopes.

## 4. Dependencies
- Python 3 standard library only (`json`, `subprocess`, `sys`, `pathlib`, `tempfile`, `os`).
