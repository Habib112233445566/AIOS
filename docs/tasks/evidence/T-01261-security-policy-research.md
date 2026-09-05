# T-01261: Package Management - Security Policy: Research

## Metadata
- **Task ID:** `T-01261`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Package Management / Security Policy
- **Status:** Complete
- **Date:** 2026-09-04

---

## 1. Research Overview & Prior Art
The AIOS system architecture requires that software package management be subject to strict security policies to prevent the introduction of untrusted, unverified, or vulnerable software into the root filesystem.

### Authoritative Upstream Standards
1. **The Update Framework (TUF) Specification (IETF draft-ietf-asdf-tuf / TUF v1.0.31)**:
   - Recommends cryptographic integrity verification (SHA-256), secure transport schemes, and anti-replay protections.
2. **Debian SecureApt (`apt-secure(8)`) & Debian Policy Manual**:
   - Requires SHA-256 digest validation of binary archives and repository authentication.
3. **NIST SP 800-218 (Secure Software Development Framework - SSDF)**:
   - Directs automated validation of software component provenance, banning unencrypted legacy communication protocols (e.g. `telnet`, `rsh`, `rlogin`, `nis`).
4. **AIOS Security Architecture (ADR-0035 & AI Constitution)**:
   - Tri-state policy enforcement modes (`Enforcing`, `Audit`, `Permissive`).
   - Mandatory cryptographic audit row emission into SQLite WAL ring on policy evaluation and transaction gating.

---

## 2. Facts vs. Assumptions

### Established Facts
1. **Fact**: `PackageSpec` contains metadata fields (`sha256`, `repository_url`, `dependencies`, `architecture`, `format`, `installed_size_bytes`) that can be evaluated against security rules.
2. **Fact**: `BaseImageSecurityPolicy` (`base_image_policy.rs`) and `DistroSecurityPolicy` (`distro_policy.rs`) establish a consistent pattern in `aiosh-core` using `PolicyMode` (`Enforcing`, `Audit`, `Permissive`), violation structs, and verdict reports.
3. **Fact**: Currently, `PackageStore` validates data model syntax (`PM1..PM5`) and dependency closure (`CS3`), but lacks a declarative, configurable security policy engine to ban insecure packages, mandate cryptographic hashes, or enforce repository whitelists.
4. **Fact**: No source code was modified during this research task.

### Assumptions
1. **Assumption**: `PackageSecurityPolicy` will be implemented in `code/aiosh-rust/aiosh-core/src/package_policy.rs` and exported via `lib.rs`.
2. **Assumption**: Prohibited packages by default will ban unencrypted legacy utilities (`telnet`, `rsh-client`, `rsh-server`, `rlogin`, `rexec`, `nis`, `yp-tools`).
3. **Assumption**: Policy evaluation will be supported for single package specs (`evaluate_spec`), proposed transactions (`evaluate_transaction`), and full stores (`evaluate_store`).

---

## 3. Unknowns & Decisions Needed
1. **Decision**: What policy invariants (`PP1..PP6`) should govern the Package Management security policy?
   - `PP1`: Policy configuration validation (valid modes, bounded arrays $\le 1024$).
   - `PP2`: Prohibited package blocking (prohibiting telnet, rsh, etc.).
   - `PP3`: Cryptographic checksum enforcement (requiring SHA-256 when `require_checksum = true`).
   - `PP4`: Transport protocol security (enforcing `https://` or `file://`).
   - `PP5`: Transaction pre-execution evaluation (vetting all proposed actions prior to mutation).
   - `PP6`: Tri-state mode semantics (`Enforcing` blocks; `Audit` records violations; `Permissive` warns).
2. **Decision**: Integration with CLI and MCP:
   - Operator CLI: `aiosh package policy [--check] [--json]`.
   - Autonomous Agent MCP: `aios.package.policy`.
   - Test Suite: New criterion `PM7` in `tools/test_package_suites.py`.
