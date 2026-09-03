# T-01111 — Base Image Build / Core Service: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Core Service

## 1. Core Service Architecture & Assembly Plan
- **Registry & Persistence**:
  - `ImageStore`: In-memory registry of base image manifests with atomic JSON persistence to disk (`image_store.json`).
  - Pre-seeded with reference production and container manifests (`debian-12-minimal-raw`, `debian-12-minimal-qcow2`, `debian-12-minimal-iso`, `alpine-319-container-tarball`).
- **Build Plan Engine**:
  - Computes a deterministic multi-stage `BuildPlan` from a `BaseImageManifest`:
    1. `Stage 1 - Bootstrap`: package manager, mirror URL, suite/release.
    2. `Stage 2 - Kernel & Bootloader`: kernel package, initramfs generator, bootloader (GRUB/systemd-boot).
    3. `Stage 3 - System Configuration`: hostname, fstab, machine-id, users.
    4. `Stage 4 - Artifact Generation`: filesystem creation, image packaging (`qemu-img`, `xorriso`).
- **Build Simulation & Sizing**:
  - Computes estimated uncompressed rootfs footprint, overhead, and verifies `size_budget_bytes` compliance.
- **Fail-Safe Invariants**:
  - Duplicate image registration rejected.
  - Manifest validation enforced upon ingestion and build execution.
