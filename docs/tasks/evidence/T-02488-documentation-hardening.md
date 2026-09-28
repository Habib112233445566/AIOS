# Task T-02488 Evidence: Sandbox Documentation Hardening

## Goal
Harden the Sandbox Documentation Subsystem against memory bloat, runaway query processing, control-character injection, and unexpected failure modes.

## Hardening Mechanisms Enforced
1. **Query Length Bounding**:
   - Both `get_topic` and `search` clamp incoming query inputs to `MAX_DOC_QUERY_LEN` (128 characters).
   - Oversized input strings are truncated safely without panicking.
2. **Control Character Stripping**:
   - Input strings are filtered with `!c.is_control()` to eliminate non-printable bytes, carriage returns, tabs, null bytes, and bell characters.
3. **Search Result Truncation**:
   - Search result lists are capped to at most `MAX_DOC_RESULTS` (32 items) using `results.truncate(MAX_DOC_RESULTS)` to prevent unbounded memory allocation.
4. **Standard Result Envelopes**:
   - CLI subcommands emit standard JSON error envelopes with code 2 and explicit error tags (`TOPIC_NOT_FOUND`, `EMPTY_QUERY`).
   - MCP endpoints return standard error codes (`SANDBOXDOC_ERR_NOT_FOUND`).

## Verification
- Added `test_sandbox_doc_hardening` verifying control character sanitization, 500-char query clamping, and control-character-only lookup rejection.
- All 6 tests in `test_sandbox_doc.rs` pass cleanly.
