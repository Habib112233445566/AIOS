# T-02129: CLI Surface Documentation — PEP Decision Engine

## Overview
- **Task ID**: `T-02129`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Documentation Updates
1. **CLI Reference**:
   - `docs/pep_decision_engine.md` updated with Section 6 covering all `aiosh pep` subcommands (`evaluate`, `rule-add`, `rule-list`, `rule-remove`, `status`).
2. **Copy-Pasteable Operator Examples**:
   - Practical shell snippets for evaluating requests, adding rules, listing rules, and inspecting status with `--json`.
3. **Exit Code Conventions & Error Envelope**:
   - Documented exit codes (0=Permit, 1=Deny, 2=Validation Error) and structured JSON error format.
4. **Traceability Links**:
   - Evidence links added for tasks `T-02121` through `T-02130`.
