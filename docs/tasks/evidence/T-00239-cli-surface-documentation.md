# T-00239 — Phase 0 — Release Packaging & Backup / CLI surface: Documentation

## Goal
Document the CLI surface of Release Packaging & Backup for operators and agents.

## Completion Notes
1. **README Documentation**:
   - Updated `docs/README.md` section `### Release Packaging & Backup (T-0211 - T-0239)`.
   - Replaced placeholder content with the finalized CLI syntax for `aiosh release generate` and `aiosh backup create`.
   - Included copy-pasteable examples for both CLI and MCP.
   
2. **Honest Limitations**:
   - Clearly documented the pre-existing Windows OS constraints (the ISO `genisoimage` mock, missing GNU dependencies).
   - Documented the constraints around the Rust CLI build on Windows natively failing in the `sandbox.rs` and `libc` surface due to unimplemented dependencies.
   - Preserved all size capping limitation warnings.

3. **Evidence Linking**:
   - Included a direct reference track to the evidence logs `T-00231` through `T-00239` at the end of the documentation section.

## Acceptance Criteria Verified
- [x] Docs updated with working example.
- [x] Limitations are stated, not omitted.
