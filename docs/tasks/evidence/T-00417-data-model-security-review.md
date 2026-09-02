# T-00417 — Documentation Index Control / data model: Security Review

## 1. Security Review Scope
This review evaluates the data model structures (`DocIndexEntry`, `DocIndexManifest`) and deserialization logic in `code/aiosh-rust/aiosh-core/src/doc_index.rs`.

## 2. Threat Analysis & Mitigations

### 1. In-Memory Deserialization & Payload Validation
- **Risk**: Malformed or maliciously crafted JSON payloads cause memory corruption or infinite allocation.
- **Mitigation**: Serde's strongly typed parsing bounds allocation. `DocIndexManifest::validate()` rejects empty/whitespace fields and duplicate paths before allowing manifest consumption.

### 2. Path Collision & Index Shadowing
- **Risk**: An attacker supplies multiple documentation entries pointing to the same file path to hide or shadow critical security notices.
- **Mitigation**: `DocIndexManifest::validate()` uses a strict `HashSet` to reject duplicate paths at deserialization time (`Duplicate doc index path: ...`).

### 3. Read-Only State Isolation
- **Risk**: Invoking data model deserialization triggers unintended side effects.
- **Mitigation**: The data model is entirely in-memory and immutable post-validation, requiring zero disk mutations or process executions.

## 3. Abuse Scenarios
- **Scenario: Shadowing of SECURITY.md**:
  - *Vector*: Ingesting an index that registers duplicate entries for `SECURITY.md`.
  - *Result*: Rejected immediately with validation error.

## 4. Conclusion
No security bypasses identified. The data model is robust and secure.
