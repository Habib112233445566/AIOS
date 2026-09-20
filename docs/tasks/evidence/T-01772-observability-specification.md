# Specification: Hardware Detection Observability Subsystem (T-01772)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Observability Subsystem (`aiosh-core::hardware_observability`)
- **Task**: `T-01772`
- **Scope**: Formal contract, invariants `HO1..HO6`, telemetry structures, and service integration.
- **Status**: **PASS (Specification Complete)**

---

## 2. Telemetry Contract & Types

### 2.1 `HardwareObservabilityReport` Schema
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareObservabilityReport {
    pub total_devices: usize,
    pub class_breakdown: BTreeMap<String, usize>,
    pub bus_breakdown: BTreeMap<String, usize>,
    pub driver_binding_count: usize,
    pub unbound_device_count: usize,
    pub driver_binding_rate: f64,
    pub total_attributes_count: usize,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_devices_found: Vec<String>,
    pub redacted_devices_count: usize,
    pub hostname: String,
    pub architecture: String,
    pub kernel_version: String,
    pub generated_at: String,
}
```

### 2.2 Method Signatures
```rust
impl HardwareObservabilityReport {
    /// Generates an observability report from the provided inventory and optional security policy (HO1..HO6).
    pub fn generate(
        inventory: &HardwareInventory,
        policy_opt: Option<&HardwareSecurityPolicy>,
    ) -> Self;
}

impl HardwareService {
    /// Discovers host hardware and generates an observability report.
    pub fn generate_observability_report(
        &self,
        policy_opt: Option<&HardwareSecurityPolicy>,
    ) -> Result<HardwareObservabilityReport, HardwareError>;
}
```

---

## 3. Observability Invariants (HO1..HO6)

- **HO1 (Total Device & Class Parity)**:
  $$\text{total\_devices} = \sum_{c \in \text{class\_breakdown}} c.\text{count}$$
- **HO2 (Bus Breakdown Parity)**:
  $$\text{total\_devices} = \sum_{b \in \text{bus\_breakdown}} b.\text{count}$$
- **HO3 (Driver Binding Accounting)**:
  $$\text{total\_devices} = \text{driver\_binding\_count} + \text{unbound\_device\_count}$$
- **HO4 (Driver Binding Rate Consistency)**:
  $$\text{driver\_binding\_rate} = \begin{cases} 0.0 & \text{if total\_devices} = 0 \\ \frac{\text{driver\_binding\_count}}{\text{total\_devices}} & \text{otherwise} \end{cases}$$
  The rate is guaranteed to be bounded in $[0.0, 1.0]$.
- **HO5 (Policy Telemetry Consistency)**:
  When a policy is provided, `policy_compliant_count` records devices with zero fatal or non-fatal violations, and `prohibited_devices_found` contains deterministically sorted IDs of prohibited hardware.
- **HO6 (Deterministic Canonical Serialization)**:
  All maps (`class_breakdown`, `bus_breakdown`) use `BTreeMap` ensuring alphabetical key ordering in serialized JSON. Roundtrip deserialization preserves all values losslessly.

---

## 4. Error Handling & Edge Cases
- **Empty Inventories**: Returns zero device counts, 0.0 binding rate, empty breakdowns, and 0 violations without panicking.
- **Unbound Devices**: Devices with `driver: None` are properly counted under `unbound_device_count`.
- **Audit Effects**: When requested via `HardwareService`, telemetry generation is logged in the SQLite WAL audit ring.
