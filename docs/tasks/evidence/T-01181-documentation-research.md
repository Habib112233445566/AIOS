# T-01181: Base Image Build Documentation Research

**Date:** 2026-09-04  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Base Image Build / Documentation  
**Task ID:** T-01181  

## 1. Executive Summary & Objective
Task `T-01181` establishes facts, architectural constraints, authoritative prior art, and concrete documentation requirements for the **Base Image Build** subsystem. This research prepares for the creation of `docs/base_image_build.md` and related documentation assets spanning the entire epic (data models, store registry, 4-stage build planning, configuration, automated integration tests, security policy, and observability telemetry).

## 2. Existing Codebase Audit & Assets

### Implemented Subsystems
1. **Data Model (`code/aiosh-rust/aiosh-core/src/base_image.rs`)**:
   - Manifest definitions: `BaseImageManifest`, `TargetFormat` (`raw`, `qcow2`, `iso`), `InitSystem`, `CompressionLevel`, `KernelConfig`.
   - Build synthesis definitions: `BuildPlan`, `BuildStage` (`Bootstrap`, `Customize`, `Package`, `Verify`), `BuildTask`.
2. **Core Service & Registry (`code/aiosh-rust/aiosh-core/src/base_image_service.rs`)**:
   - `ImageStore` with JSON persistence, idempotent manifest insertion, lookup by ID, filtering by format and distribution, and deterministic 4-stage build plan synthesis.
3. **Configuration (`code/aiosh-rust/aiosh-core/src/base_image_config.rs`)**:
   - `ImageBuildConfig` implementing resolution precedence: explicit file > environment variables (`AIOS_IMAGE_*`) > safe defaults.
   - Invariants `CF1..CF6` enforcing non-empty paths, strict printable ASCII sanitization, and numerical range constraints.
4. **Integration Test Battery (`code/aiosh-rust/aiosh-core/tests/test_base_image_automated.rs`)**:
   - Deterministic test cases `T1..T7` exercising multi-run plan stability, stress queries, precedence order, malformed payload rejections, and RAII directory cleanup.
5. **Security Policy Engine (`code/aiosh-rust/aiosh-core/src/base_image_policy.rs`)**:
   - `BaseImageSecurityPolicy` enforcing invariants `P1..P7`: dangerous kernel parameter blacklist, unencrypted legacy package blacklist, arch whitelist, filesystem whitelist, mandatory package baseline, and input poisoning guards.
   - Modes: `Enforcing`, `Audit`, and `Permissive`.
6. **Observability Telemetry (`code/aiosh-rust/aiosh-core/src/base_image_observability.rs`)**:
   - `BaseImageObservabilityReport` tracking total images, categorical distributions, policy compliance, storage budgets, and kernel inventories.
   - Validates mathematical invariants `OB1..OB5` with strict map key capacity ceilings.
7. **Production Surfaces**:
   - CLI Subcommands in `aiosh-cli`: `image list`, `image show`, `image plan`, `image filter`, `image config`, `image policy`, `image report`.
   - MCP Tools in `aiosh-mcp`: `aios.image.list`, `aios.image.get`, `aios.image.plan`, `aios.image.config`, `aios.image.policy`, `aios.image.report`.
8. **Test Automation**:
   - `tools/test_image_suites.py` validating criteria `B1..B8`.

## 3. Authoritative Prior Art & Standards

1. **Reproducible Builds Specification (reproducible-builds.org)**:
   - Requirement: Given identical source manifests and environment configurations, image artifact generation must produce bit-for-bit identical outputs (`SOURCE_DATE_EPOCH` stabilization, fixed file sorting, deterministic filesystem timestamps).
2. **Debian `debootstrap` & Rootfs Generation Tooling**:
   - Two-stage bootstrapping pattern: initial package download/unpacking followed by in-chroot configuration scripts (`chroot`, `dpkg --configure -a`).
3. **OCI Image Specification (v1.0.0 / v1.1.0)**:
   - Content-addressable layer tarballs, immutable SHA-256 digest manifests, and configuration descriptor conventions.
4. **Linux Kernel Hardening Guidelines**:
   - Upstream `Documentation/admin-guide/kernel-parameters.txt`.
   - Mandatory enablement of kernel address space layout randomization (`kaslr`), page table isolation (`pti`), and standard CPU vulnerability mitigations; absolute prohibition of disabling security parameters in production images.
5. **NIST SP 800-190 (Application Container Security Guide)**:
   - Minimizing base image attack surface, removing extraneous legacy services (e.g. `telnet`, `rsh`), and verifying image supply-chain provenance.

## 4. Facts vs. Assumptions

### Facts (Empirically Verified in Codebase)
- All manifest IDs and target format strings are strictly validated against control character and null-byte injection.
- Build plans consist of exactly 4 sequential stages (`Bootstrap` $\to$ `Customize` $\to$ `Package` $\to$ `Verify`).
- Every CLI subcommand and MCP tool invocation records an immutable hash-chained audit row to SQLite WAL.
- All test criteria `B1..B8` execute and pass via `tools/test_image_suites.py`.
- Documentation standards are enforced by `tools/check_task_docs.py` (C1..C6).

### Assumptions
- Operating agents will invoke `aios.image.*` tools via standard JSON-RPC 2.0 frames over `aiosh-mcp`.
- Human operators will manage local builds via `aiosh image *` subcommands.
- Local disk storage will contain the JSON manifest store at `/var/lib/aios/images` or an explicitly provided store path.

## 5. Decisions Needed for Implementation
1. **Target Documentation Artifact**: Should documentation live in a dedicated guide `docs/base_image_build.md` linked from `docs/README.md` (similar to `docs/distro_selection.md`)?
   - *Decision*: Yes. A comprehensive, dedicated `docs/base_image_build.md` provides in-depth technical documentation while keeping `docs/README.md` clean and navigable.
2. **CLI / MCP Example Completeness**: Should the documentation provide full, copy-pasteable JSON payloads and shell invocations for all 7 subcommands / MCP tools?
   - *Decision*: Yes. Each tool must have an exact JSON-RPC and CLI example with realistic outputs.
3. **Diagrams & Tables**: Use GitHub-compatible markdown tables and Mermaid workflow diagrams to illustrate the 4-stage build lifecycle and configuration precedence.
   - *Decision*: Yes. Visualizing the build lifecycle ensures rapid onboarding for both operators and autonomous agents.
