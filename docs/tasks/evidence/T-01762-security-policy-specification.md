# T-01762: Hardware Detection — Security Policy Specification

## Metadata
- **Task ID**: `T-01762`
- **Sub-Epic**: Sub-Epic 7: Hardware Detection Security Policy
- **Epic**: Phase 1 — Linux Base System & Bootable Target / Hardware Detection
- **Phase**: Phase 1 — Linux Base System & Bootable Target
- **Date**: 2026-09-20
- **Auditor / Implementation Engineer**: Antigravity Autonomous Agent

---

## 1. Formal Invariants (HSEC1..HSEC5)

| Invariant | Name | Formal Guarantee | Verification Criteria |
| :--- | :--- | :--- | :--- |
| **HSEC1** | Precedence Order | $\text{Deny} \succ \text{Allow} \succ \text{Default}$; fatal violation in `Enforcing` mode $\implies \text{verdict} = \text{"deny"}$. | Prohibited ID / disallowed class negative tests. |
| **HSEC2** | Attribute Redaction | $\text{redact} == \text{true} \land k \in \{\text{"address"}, \text{"mac"}, \text{"serial"}, \text{"uuid"}, \text{"wwid"}\} \implies v = \text{"<REDACTED>"}$. | Redaction unit tests. |
| **HSEC3** | Class & Bus Gatekeeping | $d.\text{class} \in \text{disallowed} \lor d.\text{bus} \in \text{disallowed} \implies \text{violation}(\text{fatal})$. | Prohibited class/bus test fixtures. |
| **HSEC4** | Determinism | $P_1 == P_2 \land I_1 == I_2 \implies \text{evaluate}(P_1, I_1) == \text{evaluate}(P_2, I_2)$. | Evaluation report equality tests. |
| **HSEC5** | Fail-Safe Defaults | Default policy enforces `Enforcing` mode, sensitive attribute redaction, and bounded inventory limits ($\le 10,000$). | Default policy tests. |

---

## 2. Policy Data Contracts

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwarePolicyMode {
    Enforcing,
    Audit,
    Permissive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareSecurityPolicy {
    pub mode: HardwarePolicyMode,
    pub disallowed_classes: Vec<DeviceClass>,
    pub disallowed_buses: Vec<DeviceBus>,
    pub prohibited_device_ids: Vec<String>,
    pub allowed_vendor_ids: Option<Vec<String>>,
    pub redact_sensitive_attributes: bool,
    pub max_devices_allowed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwarePolicyViolation {
    pub rule_id: String,
    pub device_id: String,
    pub description: String,
    pub fatal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwarePolicyReport {
    pub verdict: String,
    pub mode: HardwarePolicyMode,
    pub violations: Vec<HardwarePolicyViolation>,
    pub devices_evaluated: usize,
    pub devices_redacted: usize,
}
```

---

## 3. Evaluation API
```rust
impl HardwareSecurityPolicy {
    pub fn evaluate(&self, inventory: &HardwareInventory) -> HardwarePolicyReport;
    pub fn apply_and_sanitize(&self, inventory: &mut HardwareInventory) -> HardwarePolicyReport;
}
```
