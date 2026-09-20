# Task Evidence: T-01961 (System Update / security policy: Research)

## 1. Objective
Establish facts, constraints, prior art, and security principles for the AIOS System Update Security Policy Subsystem (Sub-Epic 7, tasks T-01961 through T-01970).

## 2. Fact vs. Assumption Analysis

### Facts (Codebase & Authoritative Standards)
1. **Existing Policy Architecture in AIOS**:
   - `aiosh-core` implements strict policy models across subsystems (`network_policy.rs`, `hardware_policy.rs`, `kernel_module_policy.rs`, `base_image_policy.rs`, `service_policy.rs`, `session_policy.rs`).
   - Every policy module exposes:
     - Policy execution modes: `Enforcing`, `Audit`, `Permissive`.
     - Standardized violation records (`PolicyViolation`: `rule_id`, `target`, `description`, `fatal`).
     - Evaluation reports (`PolicyReport`: `verdict`, `mode`, `violations`, metrics).
     - Input path hygiene (`validate_policy_path`: $\le 1024$ bytes, no `..`, no control chars, non-empty UTF-8).
     - File size cap (`MAX_POLICY_FILE_BYTES = 1_048_576` / 1 MB) and atomic persistence.
     - Error prefixes (e.g. `UPOL_VALIDATION_ERROR`, `UPOL_IO_ERROR`, `UPOL_PATH_ERROR`).
2. **Current System Update State**:
   - Core data model (`system_update.rs`) and service (`system_update_service.rs`) validate basic structural validity and SHA-256 digests.
   - However, high-level governance rules (channel restrictions, downgrade prevention, Ed25519 signature enforcement against a trusted key store, mandatory partition target requirements, and version revocation) are not yet encapsulated in a dedicated policy engine.
3. **Upstream Standards & Prior Art**:
   - **The Update Framework (TUF / RFC 8758)**: Establishes principles for surviving key compromises, preventing rollback attacks, and enforcing multi-key thresholds.
   - **NIST SP 800-193 (Platform Firmware Resiliency Guidelines)**: Specifies Section 4.1 "Protection": updates must be cryptographically authenticated, authorized for the specific device model, and strictly monotonic (preventing rollback to vulnerable builds).
   - **Android Verified Boot (AVB 2.0) & ChromeOS**: Enforces rollback index monotonic increments and rejects boot/staging of images with rollback index $< \text{current}$.
   - **Semver / Version Monotonicity**: Semantic version parsing with major/minor/patch tuple comparison to detect downgrade attempts.

### Assumptions
1. Semver comparison can be implemented safely using a deterministic token parser without pulling in heavy external crates.
2. The security policy evaluator will be decoupled from the raw service execution, allowing offline policy checks (e.g. CLI pre-flight validation, MCP policy inspection) as well as gating runtime updates.
3. Policy reports must serialize to canonical JSON for cross-substrate consumption by Python and MCP tools.

## 3. Decisions & Policy Invariants Identified (UPOL1 - UPOL6)
- `UPOL1`: Channel Authorization (manifest channel must exist in `allowed_channels`).
- `UPOL2`: Cryptographic Signature Enforcement (if `require_signature` is true, manifest must possess a signature matching `trusted_public_keys`).
- `UPOL3`: Anti-Rollback / Downgrade Prevention (if `disallow_downgrades` is true, manifest version must be $\ge$ running version).
- `UPOL4`: Partition Target Allowlist (all artifacts must target authorized partitions in `allowed_partition_targets`, and any `required_partition_targets` must be present).
- `UPOL5`: Resource Quotas (manifest total payload $\le \text{max\_payload\_bytes}$ and artifact count $\le \text{max\_artifacts\_count}$).
- `UPOL6`: Revocation Denylist (manifest version and update_id must not match any entry in `revoked_versions` or `revoked_update_ids`).

## 4. References & Citations
- The Update Framework (TUF) Specification: `https://theupdateframework.io/specification/latest/`
- NIST Special Publication 800-193: "Platform Firmware Resiliency Guidelines" (Dec 2018)
- Android Verified Boot 2.0 Specification: `https://android.googlesource.com/platform/external/avb/+/master/README.md`
- RFC 8758: "The Update Framework (TUF)"
