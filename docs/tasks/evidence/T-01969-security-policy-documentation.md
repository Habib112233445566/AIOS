# Task Evidence: T-01969 (System Update / security policy: Documentation)

## Summary
Authored Section 10 ("System Update Security Policy Subsystem") in `docs/system_update.md`.
The documentation covers:
1. **Overview & Policy Architecture**: `SystemUpdateSecurityPolicy`, execution modes (`Enforcing`, `Audit`, `Permissive`), violation records, and evaluation reports.
2. **Invariants Enforced (UPOL1 - UPOL6)**:
   - `UPOL1`: Channel Authorization against `allowed_channels`.
   - `UPOL2`: Cryptographic Signature Enforcement against `trusted_public_keys`.
   - `UPOL3`: Anti-Rollback & Downgrade Prevention via Semver parsing.
   - `UPOL4`: Partition Target Governance (allowed targets allowlist and required target presence).
   - `UPOL5`: Resource & Quota Caps (max payload bytes 1MB..10GB and artifact count 1..32).
   - `UPOL6`: Revocation Denylisting for compromised versions and update IDs.
3. **Execution Commands**: Copy-pasteable test execution commands for Rust unit test and Python smoke suite.
4. **Configuration Example**: Complete JSON configuration sample (`update_policy.json`).
5. **Constraints & Known Limitations**: Key store capacity caps (32 keys, 1024 revoked entries), path hygiene constraints, and symlink rejection.
6. **Evidence Artifacts**: Direct links to all Sub-Epic 7 evidence files.
