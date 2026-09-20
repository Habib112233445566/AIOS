# Hardening Summary: T-02088 (documentation: Hardening)

- **Target Components**: `code/aiosh-rust/aiosh-core/src/capability_doc.rs`, `code/aiosh-rust/aiosh-mcp/src/main.rs`
- **Mitigations**:
  - Category normalization: `trim().to_ascii_lowercase()` prevents casing mismatch errors.
  - Control character rejection: Pre-trim detection stops hidden control characters.
  - UTF-8 char boundary checks: Prevents slicing panics.
- **Verification**: All unit tests and smoke tests passing.
