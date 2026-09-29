# Research: T-02521 Privilege Escalation Prevention CLI Surface

- **Task**: `T-02521`
- **Sub-Epic**: Privilege Escalation Prevention / CLI surface
- **Date**: 2026-09-29
- **Status**: PASSED

## 1. Objectives & Context
The Privilege Escalation Prevention CLI Surface provides operators, administrators, and interactive agent loops with CLI commands to inspect privilege contexts, request grant-backed dynamic privilege elevations, safely de-escalate privilege tiers, and verify assigned capabilities.

## 2. Invariants & Prior Art
1. **Audit Gating & Emittance**:
   - Every CLI execution must emit a hash-chained audit record via `classify_and_emit` or `emit()`, recording actor, target tier, requested capabilities, and result.
2. **Terminal Output Sanitization**:
   - Terminal outputs must sanitize escape sequences (CWE-150 defense) unless structured `--json` output is selected.
3. **Fail-Closed Exit Codes**:
   - Return exit code `0` on successful operation or query.
   - Return exit code `1` on invalid arguments, denied escalation, or missing actor contexts.
4. **Command Surface**:
   - `aiosh privilege status [--actor A] [--json]`
   - `aiosh privilege elevate --to <tier> [--grant G] [--actor A] [--caps C1,C2] [--json]`
   - `aiosh privilege drop --to <tier> [--actor A] [--json]`
   - `aiosh privilege revoke [--actor A] [--json]`
   - `aiosh privilege check --cap <capability> [--actor A] [--json]`
   - `aiosh privilege list [--json]`

## 3. Conclusion
Ready for specification (`T-02522`).
