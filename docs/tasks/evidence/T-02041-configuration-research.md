# Task Evidence: T-02041 (Capability Model / configuration: Research)

## Task Information
- **Task ID**: T-02041
- **Title**: Capability Model / configuration: Research
- **Epic**: Phase 2 — Security Kernel & PEP Fabric / Capability Model
- **Sub-Epic**: Sub-Epic 5: Configuration Subsystem
- **Status**: Completed
- **Date**: 2026-09-20

## Summary of Research
1. **Background & Architecture Alignment**:
   - Researched existing configuration patterns across `aiosh-core`:
     - Examined `secrets_config.rs`, `distro_config.rs`, `base_image_config.rs`.
     - Standard patterns identified:
       - `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
       - Sensible `Default` implementation.
       - Methods: `from_json(&str)`, `to_json(&self)`, `from_path(&Path)`, `from_env()`, `validate(&self)`.
       - Safe size limits when reading from disk (e.g. `MAX_CONFIG_BYTES = 64 * 1024`).
       - Path validation preventing directory traversal (`..`) and control characters.
   - Identified integration points in `CapabilityService` (`aiosh-core/src/capability_service.rs`):
     - Currently hardcodes `MAX_CAPABILITY_STORE_SIZE = 10_485_760` and `MAX_CAPABILITIES_IN_REGISTRY = 10_000`.
     - Storage path is currently an optional `PathBuf`.
     - Needs a formal `CapabilityConfig` struct that governs store path, capacity, file size limit, monotonic attenuation, default expiry, and auto-pruning.

2. **Configuration Requirements**:
   - `version: String` (e.g. "1.0.0", max 32 chars).
   - `store_path: PathBuf` (defaults to `.aios/capability_store.json`).
   - `max_store_bytes: u64` (default 10 MB, valid 1 KiB .. 100 MiB).
   - `max_capabilities: usize` (default 10,000, valid 1 .. 1,000,000).
   - `default_expires_secs: Option<u64>` (optional default expiration in seconds, valid 1 .. 315,360,000).
   - `enforce_strict_monotonic: bool` (default true).
   - `auto_prune_on_load: bool` (default true).

3. **Environment Variable Strategy**:
   - `AIOS_CAPABILITY_CONFIG`: Optional path to JSON configuration file.
   - `AIOS_CAPABILITY_STORE_PATH`: Override store path.
   - `AIOS_CAPABILITY_MAX_CAPABILITIES`: Override capacity.
   - `AIOS_CAPABILITY_MAX_STORE_BYTES`: Override max store size.

4. **Conclusion**:
   - The design is consistent with all existing `aiosh-core` configuration models and integrates seamlessly with `CapabilityService`.
