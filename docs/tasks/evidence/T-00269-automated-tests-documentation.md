# T-00269 — Automated Tests: Documentation

## Documentation Updates

**File Modified**: `docs/README.md`

**Updates Made**:
1. Added an **Automated Tests** subsection under **Release Packaging & Backup** explaining the test scope and substrate distribution.
2. Included a **copy-pasteable example** showing how to invoke the Rust test suite specifically for release testing (`cargo test -p aiosh-core release`).
3. Maintained the **Known Limitations**, linking constraints on Windows execution back to the automated test skips present in the Python CLI wrapper.

## Acceptance Validation
- The documentation correctly reflects the tests implemented in `T-00264`.
- The example command runs successfully in the CLI.
- Limitations are accurately stated, preserving the honest-reporting ethos of the project.
