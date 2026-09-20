# Hardening Evidence: Hardware Detection Security Policy Subsystem (T-01768)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Security Policy Subsystem (`code/aiosh-rust/aiosh-core/src/hardware_policy.rs`)
- **Task**: `T-01768`
- **Scope**: Hardening against DoS, unbounded I/O, path traversal, list inflation, and case-sensitivity bypasses.
- **Status**: **PASS (All hardening controls implemented and verified)**

---

## 2. Hardening Controls Implemented

### 1. Bounded File Size (`MAX_POLICY_FILE_BYTES = 1 MB`)
- Enforced `MAX_POLICY_FILE_BYTES = 1_048_576` in `load_from_path()`.
- Pre-read metadata inspection ensures oversized files are rejected immediately before reading into memory, neutralizing memory exhaustion / OOM attacks.

### 2. Path Traversal & Hygiene (`validate_policy_path`)
- All policy load/save paths must pass `validate_policy_path()`:
  - Rejects empty strings.
  - Rejects paths with length > 1,024 characters.
  - Rejects control characters (`\0`, `\r`, `\n`).
  - Rejects path traversal components (`std::path::Component::ParentDir` / `..`).

### 3. List Inflation Caps (`validate`)
- Capped `prohibited_device_ids.len() <= 10_000`.
- Capped `allowed_vendor_ids.len() <= 10_000`.
- Neutralizes CPU starvation attacks from unbounded linear scans.

### 4. Case-Insensitive Vendor ID Matching
- Replaced exact set membership checks with `v.eq_ignore_ascii_case(vid)`.
- Prevents bypasses or false alarms stemming from hex case variations (e.g. `8086` vs `8086`).

### 5. Atomic Persistence
- `save_to_path()` validates the policy, creates a unique `.{name}.tmp.{pid}` sibling file, and atomically renames to the target destination.

---

## 3. Test Verification
- Authoring 4 new hardening test cases in `code/aiosh-rust/aiosh-core/tests/test_hardware_policy.rs`:
  - `test_hardening_path_traversal_rejected`
  - `test_hardening_oversized_policy_file_rejected`
  - `test_hardening_case_insensitive_vendor_matching`
  - `test_hardening_list_bound_limits`
- Total: 16/16 unit tests passing.
