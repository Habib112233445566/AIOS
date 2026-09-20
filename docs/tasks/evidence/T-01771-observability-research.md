# Observability Research: Hardware Detection Subsystem (T-01771)

## 1. Executive Summary
- **Subsystem**: Hardware Detection Observability Subsystem (`aiosh-core::hardware_observability`)
- **Task**: `T-01771`
- **Scope**: Facts, constraints, prior art, and architecture for telemetry, inventory metrics, driver binding status, and policy compliance reporting across AIOS.
- **Status**: **PASS (Research Complete)**

---

## 2. Prior Art & Authoritative Sources
1. **Linux Sysfs & Kernel Topology (`Documentation/filesystems/sysfs.txt`)**:
   - Device driver bindings are exposed via `/sys/bus/*/devices/*/driver`. A device without a symlink to a driver is unbound or unmanaged.
   - Device classifications (`class/` and PCI class codes) provide canonical categorizations (GPU, Network, Storage, Input, etc.).
2. **OpenTelemetry Hardware Metrics Semantic Conventions**:
   - Standard metrics recommend tracking device counts per class, operational status, error counts, and managed/unmanaged driver ratios.
3. **Existing AIOS Observability Subsystems**:
   - `KernelModuleObservabilityReport` (`KO1..KO6`): Summarizes loaded modules, memory, ref counts, and policy compliance.
   - `ServiceObservabilityReport` (`SO1..SO6`): Tracks service states, restart counts, dependency distributions, and policy violations.
   - `PackageObservabilityReport` (`PO1..PO6`): Tracks package counts, repository health, and compliance.

---

## 3. Facts vs Assumptions

### Facts (Empirically Verified in Codebase)
- `HardwareInventory` contains `devices: Vec<HardwareDevice>`, `summary: BTreeMap<String, usize>`, and metadata (`hostname`, `architecture`, `kernel_version`).
- `HardwareDevice` contains `class: DeviceClass`, `bus: DeviceBus`, `driver: Option<String>`, `vendor_id: Option<String>`, `device_id: Option<String>`, and `attributes: BTreeMap<String, String>`.
- `HardwareSecurityPolicy` provides `evaluate(&inventory)` returning a `HardwarePolicyReport` with `violations` and `devices_redacted`.
- BTreeMap guarantees deterministic key ordering across all substrates (Rust, Python, TypeScript).

### Assumptions & Constraints
- Hardware observability reports are generated in-memory on demand and should not require additional disk I/O beyond existing scan results.
- Driver binding rate should be represented as a bounded float $0.0 \le r \le 1.0$ rounded to 4 decimal places.
- Empty inventories must generate valid, zeroed reports without division-by-zero panics.

---

## 4. Key Design Decisions
1. **Data Model**: Introduce `HardwareObservabilityReport` in `aiosh-core::hardware_observability`.
2. **Metric Aggregations**:
   - `total_devices`: Total count of discovered devices.
   - `class_breakdown`: Count of devices per `DeviceClass`.
   - `bus_breakdown`: Count of devices per `DeviceBus`.
   - `driver_binding_count` & `unbound_device_count`: Accounting of driver-managed vs unmanaged devices.
   - `driver_binding_rate`: Floating point ratio with zero-division guard.
   - `policy_compliance`: Integrated summary of compliant devices, violations, and prohibited device IDs when a policy is provided.
3. **Deterministic Serialization**: Use `BTreeMap` and `BTreeSet` for all collections to guarantee byte-identical canonical JSON output.
