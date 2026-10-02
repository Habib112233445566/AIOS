# T-02622: Secrets Handling CLI Surface Specification

- **Task**: `T-02622`
- **Sub-Epic**: Secrets Handling / CLI surface
- **Date**: 2026-10-02
- **Status**: PASSED

## 1. Specification Deliverable
- Authored formal specification `docs/SPEC-SECRETS-CLI.md`.
- Specified command syntax, arguments, and options for:
  - `aiosh secret store`
  - `aiosh secret get`
  - `aiosh secret list`
  - `aiosh secret rotate`
  - `aiosh secret revoke`
- Defined standardized exit codes (0 = Success, 1 = Security/Operational error, 2 = Usage/Argument error).
- Defined invariants `SECCLI1` through `SECCLI6` guaranteeing safe redaction by default and zero leakage in list/error streams.
