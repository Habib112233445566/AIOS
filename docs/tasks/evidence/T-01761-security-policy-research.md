# T-01761: Hardware Detection — Security Policy Research

## Metadata
- **Task ID**: `T-01761`
- **Sub-Epic**: Sub-Epic 7: Hardware Detection Security Policy
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Research Objectives
Researched security policy architecture, Policy Enforcement Point (PEP) alignment, device allowlisting/denylisting, and sensitive attribute redaction mechanisms for the Hardware Detection subsystem.

---

## 2. Key Findings & Design Considerations

### 2.1 Hardware Security Threat Model
Host hardware discovery exposes system identifiers that may leak privacy or security data:
1. **Network MAC Addresses & UUIDs**: Hardware network MAC addresses and partition UUIDs enable cross-session tracking and reconnaissance.
2. **Rogue Peripheral Devices**: Unauthorized USB devices (e.g. BadUSB, rogue network adapters, unapproved storage) must be detectable and deniable by policy.
3. **Vendor Spoofing & Malformed IDs**: Devices reporting invalid vendor IDs or forbidden buses must be audited or filtered.

### 2.2 Policy Enforcement Modes
Aligned with AIOS policy patterns (`service_policy`, `session_policy`, `kernel_module_policy`):
- `Enforcing`: Prohibited devices are denied or stripped from the inventory; sensitive attributes are masked.
- `Audit`: All violations are recorded and flagged in evaluation reports, but devices remain visible in the inventory.
- `Permissive`: Evaluates policy without modifying inventory or blocking operations.

### 2.3 Sensitive Attribute Redaction
Attributes matching sensitive telemetry keys (`address`, `mac`, `serial`, `uuid`, `wwid`) must be redacted to `"<REDACTED>"` when `redact_sensitive_attributes` is enabled.

---

## 3. Recommended Specification for T-01762
Formulate invariants `HSEC1..HSEC5`:
- `HSEC1`: Policy Evaluation Precedence (Deny > Allow > Default).
- `HSEC2`: Sensitive Attribute Redaction (masking MACs/serials/UUIDs).
- `HSEC3`: Class and Bus Gatekeeping (filtering prohibited classes/buses).
- `HSEC4`: Deterministic Verdicts (identical inventory + policy $\implies$ byte-identical report).
- `HSEC5`: Fail-Safe Defaults (default policy enforces safe bounds and redaction).
