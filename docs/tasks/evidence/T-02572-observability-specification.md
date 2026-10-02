# T-02572: Privilege Escalation Prevention Observability Specification

- **Task**: `T-02572`
- **Sub-Epic**: Privilege Escalation Prevention / observability
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Specification Objectives
Define the contractual data model, error codes, sanitization filters, and aggregation invariants for the Privilege Escalation Prevention observability subsystem in `code/aiosh-rust/aiosh-core/src/privilege_observability.rs`.

## 2. Invariants
- `PRIVESCOBS1`: Timestamp RFC 3339 format validation.
- `PRIVESCOBS2`: Control character stripping and 256-byte length clamping.
- `PRIVESCOBS3`: 128-entry cardinality limits on outcome maps.
- `PRIVESCOBS4`: Health evaluation rules.

See complete specification in `docs/SPEC-PRIVILEGE-OBSERVABILITY.md`.
