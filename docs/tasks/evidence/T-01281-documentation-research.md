# T-01281: Package Management Documentation Research

**Date:** 2026-09-05  
**Subsystem:** Phase 1 — Linux Base System & Bootable Target  
**Component:** Package Management / Documentation  
**Task ID:** T-01281  

---

## 1. Executive Summary & Objective
Task `T-01281` establishes facts, architectural constraints, authoritative prior art, and concrete documentation requirements for the **Package Management** subsystem. This research lays the foundation for creating a comprehensive, rot-proof documentation asset (`docs/package_management.md`) and its automated verification suite (`tools/test_package_doc.py`) spanning the entire epic:
- Data models & specification invariants (PM1..PM5)
- Core service registry & transactional planning (CS1..CS5)
- Operator CLI surface (`aiosh package *`)
- Autonomous Agent MCP surface (`aios.package.*`)
- Configuration resolution & hierarchy (PC1..PC6)
- Automated lifecycle & integration testing (PT1..PT6)
- Security policy & prohibited packages (PP1..PP6)
- Observability telemetry & footprint reporting (PO1..PO6)

---

## 2. Existing Codebase Audit & Assets

### 1. Data Model (`code/aiosh-rust/aiosh-core/src/package.rs`)
- Manifest definitions: `PackageSpec`, `PackageFormat` (`deb`, `apk`, `flatpak`, `tarball`), `PackageState` (`available`, `installed`, `upgradable`, `pending_install`, `pending_removal`, `broken`).
- Transaction structures: `PackageDependency`, `PackageAction` (`Install`, `Upgrade`, `Remove`, `Reinstall`), `PackageTransaction`, `PackageQuery`.
- Invariants PM1..PM5: naming syntax (`^[a-z0-9][a-z0-9+.-]*$`), sizing bounds (version $\le 64$, desc $\le 4096$, deps $\le 256$, size $\le 100\text{ GiB}$), dependency hygiene, SHA-256 digests, state consistency.

### 2. Core Service & Registry (`code/aiosh-rust/aiosh-core/src/package_service.rs`)
- `PackageStore`: In-memory registry seeded with reference Debian 12 (`libc6`, `coreutils`, `bash`, `libssl3`, `curl`) and Alpine 3.19 (`musl`, `busybox`, `apk-tools`) packages.
- Invariants CS1..CS5: registry uniqueness, deterministic SHA-256 transaction IDs, dependency closure verification, size delta arithmetic, and atomic persistence with RAII `.tmp` cleanup.

### 3. Hierarchical Configuration (`code/aiosh-rust/aiosh-core/src/package_config.rs`)
- `PackageConfig` resolution: precedence of explicit file > environment variables (`AIOS_PACKAGE_*`) > defaults.
- Invariants PC1..PC6: store path sanity, store size bounds ($[64\text{ KiB} \dots 100\text{ MiB}]$), entity bounds ($[10 \dots 100,000]$), transport security (`https://` or `file://`), stream read cap (64 KiB).

### 4. Integration Test Suite (`code/aiosh-rust/aiosh-core/tests/test_package_automated.rs`)
- Integration matrix PT1..PT6: deterministic multi-run plan stability, lifecycle transitions, dependency closure failure modes, store limits, and delta anti-tamper rollback.

### 5. Security Policy Engine (`code/aiosh-rust/aiosh-core/src/package_policy.rs`)
- `PackageSecurityPolicy`: Modes `Enforcing`, `Audit`, `Permissive`.
- Invariants PP1..PP6: configuration bounds, case-insensitive prohibited package blocking (`telnet`, `rsh`, `rlogin`, `rexec`, `nis`, `yp-tools`), mandatory SHA-256 checksums, repository transport security, architecture/format whitelists.

### 6. Observability Telemetry (`code/aiosh-rust/aiosh-core/src/package_observability.rs`)
- `PackageObservabilityReport`: Invariants PO1..PO6 tracking inventory completeness, multi-dimensional breakdowns, footprint saturation arithmetic, bounded dependency histograms, policy compliance metrics, and deterministic JSON emission.

### 7. Operational Surfaces
- **Operator CLI (`aiosh-cli`)**: 9 subcommands under `aiosh package`: `validate`, `list`, `show`, `search`, `plan`, `apply`, `config`, `policy`, `stats`.
- **Autonomous Agent MCP (`aiosh-mcp`)**: 8 MCP tools under `aios.package.*`: `validate`, `list`, `get`, `plan`, `search`, `apply`, `config`, `policy`, `stats`, gated with PEP capability tokens and SQLite WAL audit logging.

### 8. Test Runners
- `tools/test_package_suites.py`: Validates criteria PM1..PM8.

---

## 3. Authoritative Prior Art & Standards

1. **Debian Policy Manual (§5 - Control Files & Metadata)**:
   - Package naming syntax, package relationships (`Depends`, `Pre-Depends`, `Recommends`), installed size calculation, and deterministic deb packaging.
2. **Alpine Linux Package Keeper (APK) Architecture**:
   - Lightweight package indexing, cryptographic signature chains, multi-repository overlay semantics, and fast dependency resolution.
3. **The Update Framework (TUF - CNCF Specification v1.0.33)**:
   - Resilient package repository distribution, metadata expiration, protection against rollback and replay attacks, and content-addressable digests.
4. **NIST SP 800-218 (Secure Software Development Framework - SSDF)**:
   - Safeguarding software integrity, verifying third-party software supply chains, eliminating legacy unencrypted protocols, and enforcing cryptographic validation prior to deployment.
5. **Standard Result Envelope (AIOS ADR-0035 / ADR-0036)**:
   - Uniform response envelopes (`{ "code": 0, "data": ..., "error": ... }`), fail-closed security gating, and immutable hash-chained audit logging to SQLite WAL.

---

## 4. Facts vs. Assumptions

### Facts (Empirically Verified in Codebase)
- All package names and paths are sanitized against ASCII control characters and null bytes.
- Dependency planning deterministically verifies joint dependency closure before generating a transaction plan.
- Consequential mutations and read operations write audit rows to `audit.db` via `classify_and_emit` or SQLite WAL ring via `dispatch::recorded_call`.
- Sizing totals and calculations use saturation arithmetic (`u64::saturating_add`), eliminating integer wrap-around vulnerabilities.
- All test criteria PM1..PM8 execute and pass cleanly via `tools/test_package_suites.py`.
- Documentation standards are strictly enforced by `tools/check_task_docs.py` (C1..C6).

### Assumptions
- AI agents orchestrate package actions primarily through JSON-RPC 2.0 tool calls over `aiosh-mcp`.
- System operators manage package configurations and manual inspections via `aiosh package` CLI subcommands.
- Physical unpack of deb/apk packages into rootfs filesystems will interface with lower-level disk extraction hooks in subsequent Phase 1 tasks.

---

## 5. Decisions & Requirements for Implementation

1. **Dedicated Architecture Document (`docs/package_management.md`)**:
   - Structure into 9 mandatory sections matching the established repository documentation standard:
     1. Executive Overview & Architectural Role
     2. Core Data Model & Types (PM1..PM5)
     3. Core Service, Store Registry & Transaction Lifecycle (CS1..CS5)
     4. Configuration Subsystem (PC1..PC6)
     5. Security Policy Subsystem (PP1..PP6)
     6. Observability Telemetry Subsystem (PO1..PO6)
     7. Operator CLI Surface Reference (`aiosh package *`)
     8. Autonomous Agent MCP Tool Surface Reference (`aios.package.*`)
     9. Failure Modes, Error Envelopes, and Audit Trail
2. **Automated Documentation Unit Test (`tools/test_package_doc.py`)**:
   - Validate doc existence, size bounds ($[1000 \dots 5\text{ MiB}]$), all 9 required section headers, and zero forbidden rot markers (`TODO`, `FIXME`, `TBD`, `XXX`, `PLACEHOLDER`).
3. **Integration into Master Suite (`tools/test_package_suites.py`)**:
   - Add criterion `PM9` (`test_package_doc`), expanding the verified package matrix to `PM1..PM9`.
