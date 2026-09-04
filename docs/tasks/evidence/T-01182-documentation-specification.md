# T-01182: Base Image Build Documentation Specification

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Documentation  
**Task ID:** T-01182  

## 1. Specification Overview
This document formally specifies the structural layout, interface contracts, error envelopes, audit effects, and validation constraints for the comprehensive Base Image Build documentation artifact: `docs/base_image_build.md`.

## 2. Document Structure & Content Contracts

The generated guide `docs/base_image_build.md` must adhere to the following 9-section architecture:

### Section 1: Executive Overview & Architectural Role
- Context: Role within AIOS Phase 1 (Linux Base System & Bootable Target).
- Key responsibilities: Deterministic, reproducible OS artifact generation for ethical hacking workloads, sandboxed agent execution, and QEMU/hardware bootable targets.

### Section 2: Core Data Model & Types
- Types defined in `code/aiosh-rust/aiosh-core/src/base_image.rs`:
  - `BaseImageManifest`: `id`, `distro_id`, `version`, `architecture`, `format`, `kernel`, `packages`, `filesystem`, `size_budget_bytes`, `created_at`.
  - `TargetFormat`: `raw`, `qcow2`, `iso`.
  - `KernelConfig`: `version`, `parameters`, `modules`.
  - `BuildPlan`: `image_id`, `created_at`, `stages` (ordered vector of `BuildStage`).
  - `BuildStage`: `name` (`Bootstrap`, `Customize`, `Package`, `Verify`), `status`, `tasks`.

### Section 3: 4-Stage Reproducible Build Lifecycle
1. **Stage 1 — Bootstrap**:
   - Downloads/extracts baseline rootfs packages (`debootstrap`, `apk`, `pacstrap`).
   - Verifies cryptographic signatures against upstream distribution keyrings.
2. **Stage 2 — Customize**:
   - Configures networking, `/etc/fstab`, user credentials, systemd service units.
   - Injects security hardening (kernel command line, sysctl flags, Landlock/seccomp profiles).
3. **Stage 3 — Package**:
   - Converts rootfs into destination format (`raw`, `qcow2`, or bootable hybrid `iso`).
   - Applies compression with configured compression level (`1..22`).
4. **Stage 4 — Verify**:
   - Computes deterministic SHA-256 artifact digests.
   - Asserts size budget constraints (`actual_size <= size_budget_bytes`).

### Section 4: Configuration Subsystem (`ImageBuildConfig`)
- Resolution Precedence:
  1. Explicit configuration file (`--config <path>` or local `./image_build.json`).
  2. Environment variables (`AIOS_IMAGE_BUILD_DIR`, `AIOS_IMAGE_TARGET_DIR`, etc.).
  3. Safe embedded defaults (`/tmp/aios-build`, `/var/lib/aios/images`, `raw`, 3600s, 10 GiB, level 6).
- Invariants `CF1..CF6`: Non-empty paths, ASCII printable character whitelist, timeout bounds [10..86400s], size limits [1 MiB..100 GiB], compression [1..22].

### Section 5: Security Policy Subsystem (`BaseImageSecurityPolicy`)
- Invariants `P1..P7`:
  - `P1`: Kernel parameter blacklist (`nokaslr`, `mitigations=off`, `pti=off`, `selinux=0`, `apparmor=0`, `init=/bin/sh`).
  - `P2`: Legacy unencrypted package blacklist (`telnet`, `rsh-client`, `rsh-redone-client`, `yp-tools`, `tftp`).
  - `P3`: Architecture whitelist (`x86_64`, `aarch64`, `riscv64`).
  - `P4`: Filesystem whitelist (`ext4`, `squashfs`, `btrfs`, `erofs`, `xfs`).
  - `P5`: Mandatory package requirement (must contain at least one essential system package).
  - `P6`: Non-empty packages list and valid size budget (> 0).
  - `P7`: Input poisoning guard against malicious control characters and injection tokens.
- Modes: `Enforcing` (fail-closed, blocks plan synthesis), `Audit` (non-fatal, emits audit record), and `Permissive`.

### Section 6: Observability Telemetry Subsystem (`BaseImageObservabilityReport`)
- Report fields: `generated_at`, `total_images`, `format_breakdown`, `architecture_breakdown`, `distro_breakdown`, `policy_compliant_count`, `total_size_budget_bytes`, `average_size_budget_bytes`, `unique_kernel_versions`.
- Invariants `OB1..OB5`: Categorical breakdown sum equality, compliance ceiling, average calculation.
- Capacity limits: 16 formats, 64 archs, 256 distros, 256 kernels.

### Section 7: Operator CLI Surface Reference
- Commands:
  - `aiosh image list [--format <fmt>] [--distro <id>] [--json] [--store <path>]`
  - `aiosh image show <id> [--json] [--store <path>]`
  - `aiosh image plan <id> [--json] [--store <path>]`
  - `aiosh image filter [--format <fmt>] [--distro <id>] [--json] [--store <path>]`
  - `aiosh image config [--json] [--config <path>]`
  - `aiosh image policy [<id>] [--json] [--store <path>]`
  - `aiosh image report [--json] [--store <path>]`
- Exit Codes: `0` (Success), `1` (Not found / operational failure), `2` (Invalid argument / validation failure).

### Section 8: Autonomous Agent MCP Tool Surface Reference
- Tools:
  - `aios.image.list`: List registered base images with optional filtering.
  - `aios.image.get`: Retrieve complete manifest by ID.
  - `aios.image.plan`: Synthesize 4-stage build execution plan.
  - `aios.image.config`: Query current build configuration resolution.
  - `aios.image.policy`: Validate security policy compliance for image or registry.
  - `aios.image.report`: Aggregate full observability metrics report.

### Section 9: Audit Trail & Integrity
- All commands/tools write immutable SHA-256 hash-chained audit events to SQLite WAL (`audit.db`).
- Rot-proof documentation compliance with `tools/check_task_docs.py` (C1..C6).

## 3. Interfaces Reused vs. New
- **Reused Interfaces**: All CLI subcommands in `code/aiosh-rust/aiosh-cli`, MCP tools in `code/aiosh-rust/aiosh-mcp`, and core structures in `code/aiosh-rust/aiosh-core`.
- **New Deliverables**: Living documentation artifact `docs/base_image_build.md`, link in `docs/README.md`, and integration test validation.
