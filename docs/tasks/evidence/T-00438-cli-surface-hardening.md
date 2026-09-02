# T-00438 — Documentation Index Control / CLI surface: Hardening

## 1. Hardening Scope
This task verifies and documents the hardening mechanisms implemented in `aiosh doc` CLI commands.

## 2. Implemented Hardening Protections
1. **Defensive Argument Parsing**:
   - Explicit validation of subcommands (`show`, `check`, `search`).
   - Rejection of missing search terms with exit code 2 and helpful usage text.
2. **Deterministic Output & Error Formatting**:
   - Standard prose format uses pure ASCII output markers (`[+]`, `[-]`) preventing character set encoding crashes on Windows cp1252.
   - JSON mode format emits structured payload envelopes with `{ ok, subcommand, data | error }`.
3. **Memory & I/O Bounds**:
   - Reads bounded to 16 MiB per document.
   - Immediate deterministic resource cleanup upon exit.
4. **Audit Invariants**:
   - Writes structured audit rows even on failure paths.

## 3. Verification
- All test suites (`test_doc_cli_smoke.py`) pass without leaks or crashes.
