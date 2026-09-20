# T-01763: Hardware Detection — Security Policy Scaffold

## Metadata
- **Task ID**: `T-01763`
- **Sub-Epic**: Sub-Epic 7: Hardware Detection Security Policy
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Scaffold Summary
Scaffolded `HardwareSecurityPolicy`, `HardwarePolicyMode`, `HardwarePolicyViolation`, and `HardwarePolicyReport` in `code/aiosh-rust/aiosh-core/src/hardware_policy.rs` and registered exports in `code/aiosh-rust/aiosh-core/src/lib.rs`.

---

## 2. Types and Defaults
- **`HardwarePolicyMode`**: `Enforcing`, `Audit`, `Permissive` (Default: `Enforcing`).
- **`HardwareSecurityPolicy`**:
  - `mode`: `HardwarePolicyMode::Enforcing`
  - `disallowed_classes`: `vec![DeviceClass::Other]`
  - `disallowed_buses`: `vec![DeviceBus::Unknown]`
  - `prohibited_device_ids`: `vec![]`
  - `allowed_vendor_ids`: `None`
  - `redact_sensitive_attributes`: `true`
  - `max_devices_allowed`: `10_000`
- **`HardwarePolicyViolation`**: `rule_id`, `device_id`, `description`, `fatal`.
- **`HardwarePolicyReport`**: `verdict`, `mode`, `violations`, `devices_evaluated`, `devices_redacted`.
