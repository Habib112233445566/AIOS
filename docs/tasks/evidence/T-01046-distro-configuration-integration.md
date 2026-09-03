# T-01046 — Distro Selection & Justification / Configuration: Integration

**Date:** 2026-09-03
**Type:** Integration
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Configuration

## 1. Integration Scope & Verification
- **Core Service Integration**: Integrated `DistroStore::load_from_config(&DistroConfig)` in `aiosh_core::distro_service`.
- **CLI Integration**: Integrated `DistroConfig::from_env()` into `cmd_distro` in `code/aiosh-rust/aiosh-cli/src/main.rs`.
- **New Subcommand**: Added `aiosh distro config [--json]` subcommand displaying active configuration and provenance sources.
- **MCP Integration**: Integrated `DistroConfig::from_env()` into all 4 MCP distro tools (`aios.distro.list`, `aios.distro.show`, `aios.distro.evaluate`, `aios.distro.recommend`) when custom `store_path` is not provided.

## 2. CLI Live Verification Output
```
$ aiosh distro config
AIOS Distro Configuration:
  Store Path:               config/distros.json
  Pinned Reference ID:      debian-12-minimal-x86_64
  Min Recommendation Score: 0.75
  Auto Evaluate:            true
  Weights:                  binary=0.40, security=0.30, footprint=0.30

$ aiosh distro config --json
{
  "auto_evaluate": { "source": "file", "value": true },
  "min_recommendation_score": { "source": "file", "value": 0.75 },
  "pinned_reference_id": { "source": "file", "value": "debian-12-minimal-x86_64" },
  "store_path": { "source": "file", "value": "config/distros.json" },
  "weights": { "source": "file", "value": { "binary_compatibility": 0.4, "footprint": 0.3, "security": 0.3 } }
}
```
