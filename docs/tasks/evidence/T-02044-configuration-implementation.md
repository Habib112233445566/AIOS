# Task Evidence: T-02044 (Capability Model / configuration: Implementation)

## Task Information
- **Task ID**: T-02044
- **Title**: Capability Model / configuration: Implementation
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Implementation
1. **Full Implementation in `code/aiosh-rust/aiosh-core/src/capability_config.rs`**:
   - `CapabilityConfig` struct with full serde attributes:
     - `version: String`
     - `store_path: PathBuf`
     - `max_store_bytes: u64`
     - `max_capabilities: usize`
     - `default_expires_secs: Option<u64>`
     - `enforce_strict_monotonic: bool`
     - `auto_prune_on_load: bool`
   - Complete constructors and parsers:
     - `default()`: Sensible secure defaults (10 MB store limit, 10,000 capacity, strict monotonic true, auto-prune true).
     - `from_json(&str)`: Parses and strictly validates JSON string.
     - `to_json(&self)`: Serializes validated struct to pretty-printed JSON.
     - `from_path(&Path)`: Bounded file read (64 KiB), parses and validates.
     - `from_env()`: Inspects `AIOS_CAPABILITY_CONFIG`, `AIOS_CAPABILITY_STORE_PATH`, `AIOS_CAPABILITY_MAX_CAPABILITIES`, and `AIOS_CAPABILITY_MAX_STORE_BYTES`.
     - `validate(&self)`: Enforces string lengths, bounds, control character absence, and path traversal (`..`) prevention.
     - Getters: `store_path()`, `max_store_bytes()`, `max_capabilities()`, `default_expires_secs()`, `enforce_strict_monotonic()`, `auto_prune_on_load()`.

2. **Integration with `CapabilityService` (`code/aiosh-rust/aiosh-core/src/capability_service.rs`)**:
   - Stored `config: CapabilityConfig` inside `CapabilityService`.
   - Added `with_config(mut self, config: CapabilityConfig) -> Self`.
   - Added `config(&self) -> &CapabilityConfig`.
   - Added `from_config(config: CapabilityConfig) -> Result<Self, String>` which loads existing store or initializes a new instance, auto-pruning expired capabilities if configured.
   - Updated capacity limit checks in `issue_root_capability` and `attenuate_capability` to reference `self.config.max_capabilities`.
   - Updated persistence size limit in `save_to_path` to reference `self.config.max_store_bytes`.
