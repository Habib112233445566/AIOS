# Task Evidence: T-01868 - Network Bootstrap / security policy: Hardening

## 1. Overview
- **Task ID**: `T-01868`
- **Sub-Epic**: 7 (Network Bootstrap Security Policy)
- **Goal**: Harden `NetworkSecurityPolicy` in `code/aiosh-rust/aiosh-core/src/network_policy.rs` against misuse, resource exhaustion, and failure modes.

---

## 2. Hardening Measures Implemented

1. **Structured Error Codes**:
   - Defined standard error classifications:
     - `NPOL_VALIDATION_ERROR`: Configuration and bounds constraint breaches.
     - `NPOL_IO_ERROR`: Filesystem access, metadata query, or atomic rename failures.
     - `NPOL_PARSE_ERROR`: Malformed JSON or schema serialization/deserialization failures.
     - `NPOL_PATH_ERROR`: Path hygiene violations (traversal, length > 1024, control characters).

2. **Guaranteed Tempfile Cleanup with Drop Guard**:
   - Implemented RAII `TempFileGuard` inside `save_to_path()`. If writing or atomic renaming returns early on error, the sibling temporary file `.{name}.tmp.{pid}` is deterministically removed from disk upon scope exit.

3. **Whitespace Normalization & Name Matching**:
   - In `evaluate()`, interface names and DNS server strings are trimmed before checking against `prohibited_interface_names`, `allowed_interface_names`, and `disallowed_dns_servers`, preventing evasion via leading/trailing whitespace.

4. **Address Masking Hardening**:
   - In `apply_and_sanitize()`, added IPv6 address masking (`{prefix}:xxxx::xxxx`) alongside IPv4 (`{prefix}.xxx`) and MAC address masking (`{oui}:xx:xx:xx`).

---

## 3. Verification
Executed: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_network_policy`
Result: 14 passed; 0 failed; 0 ignored; finished in 0.02s.
Exit code: 0.
