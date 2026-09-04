# T-01171 — Base Image Build / Observability: Research

**Date:** 2026-09-04
**Type:** Research
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Observability

## 1. Executive Summary & Objective
The Base Image Build subsystem requires an observability layer to inspect store health, target image distributions, format metrics, and security policy compliance without inspecting raw disk images or executing external build tools.

## 2. Facts vs Assumptions
### Facts (Observed in Codebase):
1. **Existing Base Image Subsystem**: Includes `BaseImageManifest` (`aiosh-core::base_image`), `ImageStore` (`aiosh-core::base_image_service`), `ImageBuildConfig` (`aiosh-core::base_image_config`), and `BaseImageSecurityPolicy` (`aiosh-core::base_image_policy`).
2. **Prior Art in Distro Observability**: `DistroObservabilityReport` (`aiosh-core::distro_observability`) provides aggregate metrics, categorical breakdowns (family, arch), compliance metrics, and invariant validation (`O1..O4`).
3. **Audit Integration**: Telemetry/observability queries should write audit records to the SHA-256 hash-chained SQLite WAL ring via `classify_and_emit` (CLI) and `dispatch::recorded_call` (MCP).

### Assumptions:
1. `BaseImageObservabilityReport` can be derived dynamically from `ImageStore` and `BaseImageSecurityPolicy`.
2. Breakdowns must include format (`raw`, `qcow2`, `iso`, `tarball`), architecture (`x86_64`, `aarch64`), and distro family.
3. Invariants OB1..OB5 ensure arithmetic consistency across all aggregations.

## 3. Metric & Invariant Definitions (OB1..OB5)
- **OB1 (Format Sum)**: Sum of format counts equals `total_images`.
- **OB2 (Architecture Sum)**: Sum of architecture counts equals `total_images`.
- **OB3 (Distro Sum)**: Sum of distro counts equals `total_images`.
- **OB4 (Compliance Bound)**: `policy_compliant_count <= total_images`.
- **OB5 (Size Budget Consistency)**: `total_size_budget_bytes == sum(manifest.rootfs.size_budget_bytes)`.

## 4. Decisions Needed
- Create `code/aiosh-rust/aiosh-core/src/base_image_observability.rs`.
- Wire into CLI (`aiosh image status` / `aiosh image metrics`) and MCP (`aios.image.metrics` / `aios.image.report`).
- Proceed to T-01172 for formal specification.
