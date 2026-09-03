# T-01141 — Base Image Build / Configuration: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Configuration

## 1. Configuration Architecture Research
- **Configuration Entity**: `ImageBuildConfig`
- **Key Parameters**:
  - `build_dir`: Scratch build workspace directory.
  - `output_dir`: Published image artifact storage directory.
  - `default_target`: Default base image manifest ID (`debian-12-minimal-raw`).
  - `max_build_duration_secs`: Enforced timeout per build plan execution (default: 1800s / 30 min).
  - `max_artifact_size_bytes`: Maximum allowed image output size ceiling (default: 10 GiB).
  - `compression_level`: Zstandard/gzip compression level (default: 3).
- **Configuration Precedence**:
  - Config file (`.aios/image_config.json`) overrides environment variables (`AIOS_IMAGE_*`), which override safe defaults.
- **Fail-Safe Invariants**:
  - Size budget $>0$ and $\le 100 \text{ GiB}$.
  - Timeout between 10s and 86400s.
