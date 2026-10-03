# T-02651: Secrets Handling Automated Tests Research

- **Task**: `T-02651`
- **Sub-Epic**: Phase 2 — Security Kernel & PEP Fabric / Secrets Handling / automated tests
- **Date**: 2026-10-03
- **Status**: PASSED

## 1. Research Objectives
Establish coverage baselines, test vectors, failure modes, and property-based validation requirements for the automated testing suite of the Secrets Handling subsystem across Rust core, CLI, and MCP surfaces.

## 2. Prior Art & Subsystem Analysis
Across the AIOS security kernel and PEP fabric:
1. **Automated Vector Standards**: Prior security sub-epics (e.g., `test_privilege_automated.rs`, `test_pep_grant_automated.rs`, `test_sandbox_automated.rs`) organize automated test coverage into formal deterministic vectors:
   - Data model lifecycle & state transitions (`Active` -> `Rotated` -> `Revoked` -> `Expired`).
   - Scoped authorization and privilege tier gates.
   - Zero-disclosure redaction and masking invariants.
   - Resource exhaustion, capacity limits, and payload clamping.
   - Path traversal and symlink rejection defenses.
   - Concurrent reader/writer thread safety and atomic file renames.
2. **Current Secrets Handling State**:
   - `secret_data_model.rs`: Provides `SecretEntry`, `SecretKind`, `SecretScope`, `SecretState`, `SecretMetadata`, `SecretValue`.
   - `secret_service.rs`: Provides `SecretService`, vault registry, scoped access checks, rotate, revoke, atomic persistence.
   - `secret_config.rs`: Provides bounded runtime configuration (`SecretConfig`), 64 KiB ceilings, and env overrides.
   - `aiosh-cli`: Provides `aiosh secret <store|get|list|rotate|revoke|config>`.
   - `aiosh-mcp`: Provides MCP tools `aios.secret.<store|get|list|rotate|revoke|config>`.

## 3. Facts vs Assumptions
- **Fact**: Unit and smoke integration tests exist, but a comprehensive multi-vector automated integration suite (`test_secret_automated.rs`) is required to validate edge cases and concurrent operations.
- **Fact**: Redaction is non-negotiable: calling `get` without `--expose` or `expose: true` must never disclose the raw payload. Calling `list` must never leak payload data or hashes.
- **Fact**: Cross-tenant scope isolation must hold: an actor cannot access another actor's secrets unless elevated to `Admin` or `SystemKernel`.
- **Fact**: Atomic disk persistence must guarantee no file corruption or partial writes during concurrent rotation and revocation.

## 4. Key Design Decisions for Specification (T-02652)
1. Specify formal test vectors `AUTOSEC1` through `AUTOSEC8` in `docs/SPEC-SECRETS-AUTOMATED-TESTS.md`.
2. Define `test_secret_automated.rs` in `code/aiosh-rust/aiosh-core/tests/test_secret_automated.rs`.
3. Validate lifecycle, scoped gates, privilege tiers, rotation versions, revocation lockouts, and concurrency safety.
