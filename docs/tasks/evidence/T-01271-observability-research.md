# T-01271: Package Management / Observability - Research

## Executive Summary
This research document establishes the foundation, constraints, prior art, and architecture for the **Observability & Telemetry Subsystem** of AIOS Package Management.
It contrasts upstream Linux package management telemetry patterns (Debian `dpkg.log`, `apt/history.log`, Alpine `apk.log`, and Prometheus/OpenTelemetry metrics) with AIOS requirements for real-time autonomous agent supervision, security auditing, and operator inspection.

---

## 1. Upstream Prior Art & Standards Review

### 1.1 Debian & APT Logging Standards
- **`dpkg` log (`/var/log/dpkg.log`)**:
  - Format: Timestamped linear journal: `YYYY-MM-DD HH:MM:SS [status|action] <package>:<arch> <version> <target-version>`.
  - Stages: `install`, `configure`, `trigproc`, `remove`, `purge`, `status <state>`.
  - Limitations: Plain text, lacking structured aggregations, dependency graphs, or machine-readable JSON metrics.
- **APT history (`/var/log/apt/history.log` and `term.log`)**:
  - Structure: Stanzas delineated by `Start-Date`, `CommandLine`, `Install`, `Upgrade`, `Remove`, `Error`, and `End-Date`.
  - Captures transactional boundaries, requesting user/agent, and exit status.

### 1.2 Alpine Linux `apk-tools` Logging
- **`apk` log (`/var/log/apk.log`)**:
  - Records transaction installation/upgrades, size delta calculations, and hook trigger execution.
  - Minimalist; lacks runtime query API for agent consumption.

### 1.3 Cloud-Native & OpenTelemetry System Metrics
- Standard package metrics for system observability:
  - Gauge: Total packages by state (`available`, `installed`, `upgraded`, `removed`).
  - Gauge: Total package disk footprint (bytes) and average package size.
  - Gauge: Package format distribution (`deb`, `tar_gz`, `raw_binary`).
  - Gauge: Package architecture distribution (`amd64`, `x86_64`, `aarch64`).
  - Counter: Policy evaluations, compliant packages, and violation tallies.
  - Histogram / Breakdown: Dependency distribution (zero dependencies, 1-5, 6-10, 11+).

---

## 2. AIOS Internal Observability Standards & Patterns

### 2.1 Precedent in Distro & Base Image Subsystems
AIOS has established a consistent pattern across Phase 1 subsystems:
1. **`DistroObservabilityReport`** (`code/aiosh-rust/aiosh-core/src/distro_observability.rs`):
   - Computes breakdown of profiles by family, architecture, production readiness, and security policy compliance.
   - Exposed via CLI `aiosh distro stats` and MCP `aios.distro.stats`.
2. **`BaseImageObservabilityReport`** (`code/aiosh-rust/aiosh-core/src/base_image_observability.rs`):
   - Computes breakdown of images by format, architecture, distro, size budget, and policy compliance.
   - Exposed via CLI `aiosh image stats` and MCP `aios.image.stats`.
3. **`Evidence & Task Metrics`** (`code/aiosh-rust/aiosh-core/src/evidence_service.rs`):
   - Emits structured JSON metrics to SQLite WAL audit ring with sequence hash chaining (ADR-0035).

---

## 3. Fact vs. Assumption Matrix

| Category | Fact (Verified in Code / Standards) | Assumption (To be Validated in Specification) |
|---|---|---|
| **Package Store State** | `PackageStore` holds registered `PackageSpec` entries with states (`Available`, `Installed`, `Upgraded`, `Removed`), format, architecture, and byte sizes. | An observability report can be computed in $O(N)$ time directly from `PackageStore` without mutating store state. |
| **Security Policy Integration** | `PackageSecurityPolicy` evaluates package specs against rules PP1..PP6 and yields a `PackagePolicyVerdict`. | Observability reports should compute overall policy compliance counts and categorize violations without executing blocking actions. |
| **Telemetry Format** | All AIOS reports serialize to serde-compatible JSON with deterministic field ordering (`BTreeMap` for distributions). | Output will include total counts, state distributions, format distributions, architecture breakdowns, size aggregations, and policy compliance rates. |
| **Operator & MCP Access** | CLI provides `aiosh package <subcommand>` and MCP server provides `aios.package.<tool>`. | CLI subcommand `aiosh package stats [--config <path>] [--store <path>] [--json]` and MCP tool `aios.package.stats` will expose this data. |
| **Audit Logging** | Every observability inspection is a read-only query that must be logged to the PEP ring buffer (ADR-0035). | Read-only inspection does not require elevated capability grants but records an audit row with actor ID and query timestamp. |

---

## 4. Unknowns & Architectural Decisions Needed

1. **Naming & Command Verbs**:
   - *Decision*: Follow the canonical pattern established by `aiosh distro stats` / `aios.distro.stats` and `aiosh image stats` -> Use `aiosh package stats` and `aios.package.stats`.
2. **Performance on Large Stores**:
   - *Decision*: Bound iteration count to `max_entity_count` (capped at 100,000 per PC3) with saturating arithmetic to prevent overflow.
3. **Dependency Depth Aggregation**:
   - *Decision*: Provide categorical buckets (`0`, `1-5`, `6-10`, `11+`) in a `BTreeMap<String, usize>` to avoid unbounded vector sizes.
4. **Integration with Master Package Suite**:
   - *Decision*: Add criterion `PM8` (`package observability telemetry report & invariants (PO1..PO6)`) to `tools/test_package_suites.py`.

---

## 5. Citations & References
- Debian Policy Manual: Package States and Control Fields (§7.1, §7.2).
- Debian `dpkg.log(5)` & APT `apt.conf(5)` logging specifications.
- Alpine Linux `apk(8)` transaction commit telemetry.
- OpenTelemetry System Metrics Semantic Conventions (Version 1.25.0).
- AIOS Architecture Decision Record ADR-0035: Policy Enforcement Point & Audit Ring Buffer.
- AIOS Distro & Base Image Observability Models (`code/aiosh-rust/aiosh-core/src/distro_observability.rs`).
