# Task Evidence: T-01908 - System Update Mechanism / data model: Hardening

## 1. Overview
- **Task ID**: `T-01908`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Implement security hardening mitigations for the System Update Mechanism data model in `aiosh-core`.

---

## 2. Implemented Mitigations
1. **Directory Traversal & Path Hygiene (`THREAT-UPD-01`)**:
   - Added `MAX_ARTIFACT_FILENAME_LEN = 128`.
   - In `UpdateArtifact::validate()`, forbidden:
     - Directory traversal patterns (`..`).
     - Slashes (`/`) and backslashes (`\`).
     - Hidden files (names starting with `.`).
     - Whitespace and non-printable control characters.
2. **Denial-of-Service & Resource Bounds (`THREAT-UPD-05`)**:
   - Added `MAX_ARTIFACTS_PER_MANIFEST = 32`.
   - In `UpdateManifest::validate()`, enforced `artifacts.len() <= MAX_ARTIFACTS_PER_MANIFEST`.
   - In `UpdateManifest::total_bytes()`, replaced basic summing with saturating addition (`fold(0u64, |acc, a| acc.saturating_add(a.size_bytes))`), preventing integer overflow attacks.
3. **Partition & Filename Collision Defense**:
   - In `UpdateManifest::validate()`, enforced uniqueness for artifact filenames and partition targets (`HashSet`), rejecting duplicate artifacts or ambiguous multiple payloads targeting the same partition.
4. **Cross-Substrate Parity**:
   - Mirrored all validation and hardening rules in `test_system_update_smoke.py`.

---

## 3. Test Verification
- Rust unit tests: `cargo test --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core --test test_system_update` (7/7 passed).
- Python smoke tests: `python code/aiosh-cli/tests/test_system_update_smoke.py` (5/5 passed).
