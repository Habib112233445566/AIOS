# Specification: Secrets Handling Automated Tests (SPEC-SECRETS-AUTOMATED-TESTS)

- **Status**: APPROVED
- **Date**: 2026-10-03
- **Subsystem**: Phase 2 Security Kernel & PEP Fabric / Secrets Handling Automated Tests
- **Binding**: ADR-0035 §F-2

## 1. Overview
The Secrets Handling automated test suite provides deterministic verification of vault operations, scoped authorization policies, privilege tier gating, cryptographic zero-leakage redaction, and multi-threaded concurrency invariants across the AIOS security architecture.

## 2. Test Vectors

| Vector ID | Name | Description | Invariants Tested |
|:---|:---|:---|:---|
| `AUTOSEC1` | Full Secret Lifecycle | Verifies `Active` -> `Rotated` -> `Revoked` lifecycle transitions and accessibility gates | `SECSVC1`, `SECSVC2` |
| `AUTOSEC2` | Scope Boundary Enforcement | Enforces isolation between `Global`, `Actor(id)`, `Session(id)`, and `Environment(name)` scopes | `SECSVC3` |
| `AUTOSEC3` | Privilege Tier Access Gates | Enforces hierarchical privilege access (`SystemKernel`/`Admin` full access, `Operator` env access, `User` actor access, `Guest` denial) | `SECSVC4` |
| `AUTOSEC4` | Zero-Disclosure Redaction | Guarantees payload masking by default, strict `expose` gating, and zero payload leakage in `list` operations | `SECSVC5` |
| `AUTOSEC5` | Version Tracking & Fingerprinting | Validates version increments, SHA-256 fingerprint updating upon rotation, and memory zeroization upon revocation | `SECSVC6` |
| `AUTOSEC6` | Capacity & Numerical Bounds | Enforces configured limits on vault entry capacity, secret payload size, and file size limits | `SECCONF1`, `SECCONF3` |
| `AUTOSEC7` | Atomic Persistence & Tamper Rejection | Verifies atomic disk writes, symlink rejection, path traversal rejection, and corrupted JSON fail-closed behavior | `SECSVC7`, `SECCONF2` |
| `AUTOSEC8` | Multi-Threaded Concurrency Safety | Verifies thread-safety and race-free state transitions under concurrent multi-actor read/rotate operations | Kernel Thread Safety |

## 3. Test Suites & Execution

### 3.1 Rust Core Automated Test Suite
Located in `code/aiosh-rust/aiosh-core/tests/test_secret_automated.rs`:
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_secret_automated
```

### 3.2 Cross-Surface Regression Verification
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-cli secret_cli_tests
cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-mcp test_mcp_secret_tools_execution
```
