# T-01764: Hardware Detection — Security Policy Implementation

## Metadata
- **Task ID**: `T-01764`
- **Sub-Epic**: Sub-Epic 7: Hardware Detection Security Policy
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Implementation Summary
Implemented the full security policy evaluation, validation, serialization, and sanitization engine in `code/aiosh-rust/aiosh-core/src/hardware_policy.rs` and integrated it with `HardwareService` in `code/aiosh-rust/aiosh-core/src/hardware_service.rs`.

---

## 2. Invariant Enforcement (HSEC1..HSEC5)
1. **HSEC1 (Precedence Order)**:
   - Prohibited device IDs and fatal violations in `Enforcing` mode result in a `deny` verdict.
2. **HSEC2 (Attribute Redaction)**:
   - Identifies sensitive keys (`address`, `mac`, `serial`, `uuid`, `wwid`) and replaces values with `"<REDACTED>"`.
3. **HSEC3 (Class & Bus Gatekeeping)**:
   - Evaluates `disallowed_classes` and `disallowed_buses`, recording fatal violations (`HPOL-CLASS`, `HPOL-BUS`).
4. **HSEC4 (Deterministic Reports)**:
   - Violations are deterministically sorted by `rule_id` then `device_id`.
5. **HSEC5 (Fail-Safe Defaults & Persistence)**:
   - `validate()` validates limits ($1 \le \text{max\_devices} \le 50,000$, ID length $\le 256$, vendor hex format).
   - `load_from_path()` safely falls back to `default()` if file does not exist.
   - `save_to_path()` executes atomic write and rename.

---

## 3. Integration with HardwareService
- Added `HardwareService::scan_with_policy(&self, options: &HardwareScanOptions, policy: &HardwareSecurityPolicy) -> Result<(HardwareInventory, HardwarePolicyReport), String>`.
