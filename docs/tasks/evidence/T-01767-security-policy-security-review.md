# Security Review: Hardware Detection Security Policy Subsystem (T-01767)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Security Policy Subsystem (`code/aiosh-rust/aiosh-core/src/hardware_policy.rs`, `hardware_service.rs`)
- **Task**: `T-01767`
- **Scope**: Comprehensive security audit of policy definition, validation (`HSEC5`), evaluation (`HSEC1..HSEC4`), sanitization/redaction (`HSEC2`), and persistence mechanisms.
- **Verdict**: **PASS WITH HARDENING RECOMMENDATIONS** (No critical bypasses; recommendations identified for bounded I/O, case-insensitive vendor matching, and path traversal guards).

---

## 2. Threat Modeling & Abuse Scenarios

### Scenario 1: Policy File Denial of Service (DoS / OOM)
- **Vector**: An attacker or rogue process points `HardwareSecurityPolicy::load_from_path` to an unbounded file (e.g. a 10 GB file or `/dev/zero`).
- **Risk**: High memory consumption leading to OOM crash of the AIOS core process.
- **Mitigation Needed (T-01768)**: Check `metadata.len()` before reading; reject files exceeding `MAX_POLICY_FILE_BYTES = 1,048,576` (1 MB).

### Scenario 2: Path Traversal on Policy Persistence
- **Vector**: A malicious caller passes a path containing `..` or control characters to `load_from_path` or `save_to_path` (e.g. `../../etc/shadow`).
- **Risk**: Unauthorized file overwrite or information leakage.
- **Mitigation Needed (T-01768)**: Validate path hygiene: reject paths containing `..` (ParentDir), control characters (`\0`, `\n`, `\r`), or lengths exceeding 1,024 bytes.

### Scenario 3: Vendor ID Case-Sensitivity Bypass
- **Vector**: A policy permits vendor `8086`, but a device reports vendor `8086` in uppercase `8086` or lowercase. If checked case-sensitively with `.contains()`, a legitimate or malicious device could be misclassified.
- **Risk**: Inconsistent evaluation verdicts.
- **Mitigation Needed (T-01768)**: Use ASCII case-insensitive comparison for vendor IDs (`eq_ignore_ascii_case`).

### Scenario 4: Policy List Unbounded Growth (CPU DoS)
- **Vector**: A policy contains 500,000 prohibited device IDs or allowed vendor IDs.
- **Risk**: Quadratic search time during `evaluate()` causing thread starvation.
- **Mitigation Needed (T-01768)**: In `validate()`, enforce length bounds: `prohibited_device_ids.len() <= 10_000` and `allowed_vendor_ids.as_ref().map_or(0, |v| v.len()) <= 10_000`.

---

## 3. PEP Gating & Audit Verification
- All scan and policy evaluations triggered via `HardwareService::scan_with_policy` are read-only policy operations or sanitize in-memory inventories.
- State-changing mutations (e.g., saving policy) require operator privileges and atomic writes with `.tmp` and rename.
- Verified that `HardwareService` calls emit tamper-evident audit records into the SQLite WAL ring.

---

## 4. Conclusion & Hand-off to T-01768
All identified edge cases are scoped for immediate remediation in `T-01768` (Hardening).
No architectural blockers exist.
