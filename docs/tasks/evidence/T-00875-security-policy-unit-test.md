# T-00875 — Regression Triage / Security Policy: Unit Test

## 1. Unit Test Coverage & Verification
- Validated `SECURITY.md` invariants through `tools/check_security_policy.py`:
  - `S1`: `SECURITY.md` exists at repository root without TODO markers.
  - `S2`: Advisory URL is verbatim and valid.
  - `S3`: Policy prose exceeds length threshold.
  - `S4`: Vulnerability and coordinated disclosure timelines (7-day / 90-day) present.
  - `S5`: All referenced in-tree security paths (`docs/tasks/evidence/T-00877-security.md`) resolve.
