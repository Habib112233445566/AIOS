# T-01116 — Base Image Build / Core Service: Integration

**Date:** 2026-09-03
**Type:** Integration
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Test Suite Integration
- Connected criterion `B2`: `base image store registry, persistence & build plan synthesis` to `tools/test_image_suites.py`.
- Verified live runner execution:
```
[+] B1 base image data model integrity & invariant validation
[+] B2 base image store registry, persistence & build plan synthesis

PASS: image_suites criteria (B1..B2)
```

## 2. Service Exposure
- Exported `pub mod base_image_service;` from `aiosh-core`.
- `ImageStore` and `BuildPlan` ready for CLI and MCP surface integration in subsequent sub-epics.
