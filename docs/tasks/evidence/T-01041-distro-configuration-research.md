# T-01041 — Distro Selection & Justification / Configuration: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Configuration

## 1. Objectives & Scope
Research the design, data contracts, and environment resolution for the Linux Distribution Selection & Justification configuration subsystem (`aiosh_core::distro_config`).
- Provide deterministic configuration resolution for distro store path, pinned reference distro, score weighting profile, and recommendation threshold.
- Support multi-tier resolution: Explicit Path $\rightarrow$ Environment Variables (`AIOSH_DISTRO_CONFIG`, `AIOSH_DISTRO_STORE_PATH`, `AIOSH_DEFAULT_DISTRO`) $\rightarrow$ Project Root Defaults (`config/distro.json`).
- Ensure source provenance tracking (`to_json_with_sources`) to distinguish environment overrides from configuration file and compile-time defaults.
- Enforce strict size bounds (64 KiB) and input validation to protect against malicious configuration injections.

## 2. Configuration Schema & Layout
```json
{
  "store_path": "config/distros.json",
  "pinned_reference_id": "debian-13",
  "min_recommendation_score": 70.0,
  "weights": {
    "security": 0.30,
    "stability": 0.25,
    "footprint": 0.15,
    "package_availability": 0.20,
    "hardware_support": 0.10
  },
  "auto_evaluate": true
}
```

## 3. Environment Variable Precedence
1. `AIOSH_DISTRO_CONFIG`: Path to custom `distro.json` configuration file.
2. `AIOSH_DISTRO_STORE_PATH`: Overrides `store_path` pointing to distro catalog.
3. `AIOSH_DEFAULT_DISTRO`: Overrides `pinned_reference_id`.
4. Default Fallback: `config/distro.json` relative to workspace root or sensible defaults if missing.

## 4. Architectural Boundaries & Security
- **Strict Bounds**: Configuration payload read capped at 65,536 bytes (`take(65_536)`).
- **Weight Normalization**: Sum of weights must equal 1.0 $\pm$ 0.001 or be automatically normalized.
- **Path Sanitization**: Path strings normalized without arbitrary `..` traversal.
- **Fail-Safe Defaults**: If config file is missing or unparseable, `DistroConfig::default()` allows core operations to proceed with audited warning.

## 5. Next Steps
Proceed to `T-01042` (Specification) to formalize the Rust structs, error types, and validation rules.
