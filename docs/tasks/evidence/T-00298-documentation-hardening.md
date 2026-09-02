# T-00298 — Release Packaging & Backup: Documentation Hardening

## Hardening Details

- **Constraint Explicitly Documented**: A robust system documentation doesn't just describe the happy path; it explicitly sets expectations on bounds and failures to "harden" operators against misuse.
- **Resource Constraints Detailed**: We have hardened the `docs/README.md` by explicitly enumerating the protective constraints in the `Known Limitations` section.
  - The documentation clearly notes the **64KB config file limit** protecting against DoS.
  - The documentation notes the **2GB file size limit** and **symbolic link exclusion** during directory walks to prevent infinite recursion or disk saturation.
  - The documentation sets expectations on the OS layer, indicating that Windows compatibility is currently stubbed/mocked due to missing `genisoimage` dependencies.

## Acceptance Validation
- The documentation accurately and honestly reports the failure envelopes and physical boundaries of the system.
- The task is complete.
