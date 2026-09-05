# T-01251: Package Management - Automated Tests: Research

## Metadata
- **Task ID:** `T-01251`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Automated Tests
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Context & Existing Codebase Analysis
The AIOS Package Management subsystem spans five major layers:
1. **Data Model (`code/aiosh-rust/aiosh-core/src/package.rs`)**:
   - Invariants `PM1..PM5`: package name syntax, size bounds, dependency hygiene, SHA-256 checksums, and HTTPS/file repository transport.
2. **Core Service (`code/aiosh-rust/aiosh-core/src/package_service.rs`)**:
   - Invariants `CS1..CS5`: package uniqueness, deterministic SHA-256 transaction planning, dependency closure enforcement, delta arithmetic & anti-tamper checking, and atomic disk persistence.
3. **Configuration Subsystem (`code/aiosh-rust/aiosh-core/src/package_config.rs`)**:
   - Invariants `PC1..PC6`: store path validation, store size ceiling [64 KiB .. 100 MiB], entity count bounds [10 .. 100,000], repository scheme security (`https://` or `file://`), file/env precedence, and 64 KiB stream-reading bounds.
4. **Operator CLI Surface (`code/aiosh-rust/aiosh-cli/src/main.rs`)**:
   - `aiosh package validate`, `list`, `show`, `search`, `plan`, `apply`, `config`.
5. **Autonomous Agent MCP Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - `aios.package.validate`, `list`, `get`, `plan`, `search`, `apply`, `config`.

Existing verification currently consists of targeted unit suites:
- `test_package_data_model.rs` (7 tests, PM1)
- `test_package_service.rs` (9 tests, PM2)
- `aiosh test_cmd_package_flow` (PM3)
- `aiosh-mcp test_mcp_package_tools` (28 assertions, PM4)
- `test_package_config.rs` (7 tests, PM5)

**Identified Gap**:
There is currently no end-to-end automated testing suite that validates the end-to-end lifecycle cohesion across these layers (e.g. multi-step state transitions, complex dependency closure cycles, store limit breaches during mutation, and deterministic transaction plan reproducibility under scale).

---

## 2. Authoritative Sources & Prior Art
1. **Debian Policy Manual — Chapter 7 (Package Relationships)**:
   - Citation: Debian Policy v4.7.0.0 §7.1–7.4 (Dependencies, Conflicts, Breaks, Pre-Depends).
   - Invariant: A package cannot be installed or upgraded if its explicit dependency requirements cannot be satisfied within the active closure.
2. **Alpine Linux `apk-tools` Architecture**:
   - Citation: `apk-tools` v2/v3 design specifications (transaction commit graph and virtual world solver).
   - Invariant: Package installations and removals must compute a non-conflicting directed acyclic graph (DAG) before modifying disk state.
3. **Reproducible Builds Specification**:
   - Citation: Reproducible Builds initiative (deterministic hash computation over build and transaction artifacts).
   - Invariant: Identical sequence of package actions on identical initial state MUST produce bit-for-bit identical transaction hashes (`CS2`).
4. **ADR-0035 & ADR-0036**:
   - Invariant: Fail-fast, non-skipping testing law with deterministic auditability and zero silent failures.

---

## 3. Facts vs. Assumptions

### Established Facts
1. **Fact**: `PackageStore` implements deterministic transaction hashing via `CS2` using SHA-256 over sorted package action tuples.
2. **Fact**: `PackageStore::apply_transaction` enforces anti-tamper checking (`CS4`) by recomputing the plan hash before state modification.
3. **Fact**: The configuration subsystem enforces `PC2` (max store size 100 MiB) and `PC3` (max entity count 100,000).
4. **Fact**: `tools/test_package_suites.py` is the single source of truth for the Package Management test runner, currently testing criteria `PM1..PM5`.

### Assumptions
1. **Assumption**: Automated integration testing should be runnable 100% offline without connecting to external repositories. All repository sources in tests can be synthetic `https://deb.debian.org` or `file:///` URLs.
2. **Assumption**: End-to-end lifecycle testing (Install -> Upgrade -> Remove) can be validated in a dedicated integration test suite `test_package_automated.rs` testing criteria `PT1..PT5`.
3. **Assumption**: Adding criterion `PM6` to `tools/test_package_suites.py` will integrate the automated test suite into the master CI runner without breaking existing smoke suites.

---

## 4. Unknowns & Decisions Needed
1. **Decision**: What core criteria should the automated testing suite (`test_package_automated.rs`) evaluate?
   - *Proposal*:
     - `PT1`: Deterministic plan reproducibility under high iteration (50 iterations yielding identical hashes and plans).
     - `PT2`: Multi-step lifecycle state transitions (Uninstalled $\to$ Installed $\to$ Upgraded $\to$ Removed) with delta verification.
     - `PT3`: Dependency closure failure modes (cyclic dependencies, missing required dependencies, incompatible format mixing).
     - `PT4`: Configuration-governed store bounds enforcement (preventing registration exceeding `max_entity_count`).
     - `PT5`: Transaction anti-tamper and rollback integrity (tampered plan hash rejection with pristine store state preserved).
2. **Decision**: Integration in Master Runner:
   - Add criterion `PM6` to `tools/test_package_suites.py` executing `test_package_automated`.
