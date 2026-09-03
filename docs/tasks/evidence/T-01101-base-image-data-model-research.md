# T-01101 — Base Image Build / Data Model: Research

**Date:** 2026-09-03
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Subsystem Scope & Prior Art
- **Target Distribution**: `debian-12-minimal-x86_64` (selected and justified in T-01001..T-01100).
- **Image Artifact Formats**:
  - `Raw` (`.img`): Raw disk image for direct dd/flashing or loop-mounting.
  - `Qcow2` (`.qcow2`): Copy-on-write sparse format for QEMU/KVM development and CI validation.
  - `Iso` (`.iso`): Hybrid El Torito bootable ISO for bare-metal boot and UEFI/BIOS installation.
  - `Tarball` (`.tar.zst`): Rootfs tarball archive for container runtime and chroot verification.
- **Root Filesystem & Kernel Composition**:
  - Rootfs: Clean `debootstrap --variant=minbase` footprint (~150–250 MiB uncompressed).
  - Kernel: Debian LTS Linux kernel (`6.1.x`) with minimal modern initramfs (`dracut` or `initramfs-tools`).
  - Filesystem: Ext4 or Squashfs (read-only base + overlayfs writable scratch).
- **Reproducible Build Manifest**:
  - Deterministic package pin versions, package hashes, build timestamps, and SHA-256 artifact checksums.

## 2. Invariant Requirements
- `I1 (Format Validity)`: Image target format must be one of `raw`, `qcow2`, `iso`, `tarball`.
- `I2 (Identifier & SemVer)`: Image ID must be alphanumeric lowercase with hyphens, and version must follow SemVer 2.0.
- `I3 (Package List Non-Emptiness)`: Package set must include essential core packages (e.g., `systemd`, `linux-image-amd64`).
- `I4 (Size & Checksum)`: Size budget must be positive and bounded (e.g. max 4 GiB), and checksums must be valid 64-hex SHA-256 hashes when computed.
