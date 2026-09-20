# Documentation Evidence: T-02099 (recovery & validation: Documentation)

- **Target File**: `docs/capability_model.md`
- **Section Added**: Section 15 ("Capability Store Recovery, Quarantine, and Deep Validation (`CAPREC1..CAPREC6`)")
- **Content Documented**:
  - Invariants `CAPREC1..CAPREC6` formal definitions.
  - Rust types `CapabilityRecoveryAction` and `CapabilityValidationReport`.
  - Non-destructive timestamped quarantine backup (`<store>.bak.<timestamp>`).
  - Deep validation rules (cycle detection, monotonic attenuation, capacity bounds).
  - MCP JSON-RPC tools `aios.capability.recover` and `aios.capability.validate`.
- **Status**: Completed.
