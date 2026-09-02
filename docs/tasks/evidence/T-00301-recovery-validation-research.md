# T-00301 — Release Packaging & Backup: Recovery & Validation Research

## Goal
Establish facts, constraints, and prior art for the recovery and validation of Release Packaging & Backup artifacts in AIOS.

## Facts (Derived from Current State)
- **Artifact Formats**: 
  - Release packages are generated as `.iso` files via `genisoimage` (or mocked in tests). 
  - Backup snapshots are generated as `.zip` files using the native Rust `zip` crate.
- **Ledger Verification**: The system already computes and stores a SHA256 hash of the Release manifest when generating releases (in `aiosh-core/src/release.rs`). The `outcome_detail` in the `AuditRing` stores this hash.

## Validation Strategies
1. **ISO Validation**:
   - *Fact*: To validate an ISO without mounting it, standard tools like `isoinfo` (part of `cdrkit`) can be used to read the directory structure, but relying on external binaries creates cross-platform fragility (as seen with `genisoimage` on Windows). 
   - *Decision Needed*: Should AIOS natively parse ISO headers in Rust, or simply rely on the SHA256 cryptographic hash against the ledger to prove no corruption occurred post-generation? (Recommendation: Rely on cryptographic hashing + ledger parity for Phase 0 to minimize dependencies).
2. **Backup Validation**:
   - *Fact*: The `zip` crate used for creation (`zip::ZipWriter`) also provides `zip::ZipArchive` which can iterate over the archive to prove structural integrity without extracting files to disk.

## Recovery Strategies
1. **Backup Recovery (Restore)**:
   - *Fact*: A restore mechanism requires extracting a `.zip` archive over a target directory.
   - *Constraint*: Extracting a zip file blindly can introduce path traversal vulnerabilities (e.g., zip slip attacks where entries contain `../`). Any native Rust extraction logic *must* sanitize paths before writing.

## Unknowns & Decisions Needed
1. **CLI Surface for Validation**: Should we expose `aiosh backup validate <file>` or just embed validation immediately after generation? (Recommendation: Expose explicit CLI commands so operators can validate cold storage).
2. **Restore Overwrite Policy**: When restoring a backup, should the system aggressively overwrite existing files, or fail if the directory is not empty? (Recommendation: Fail if the target directory is not empty, forcing the operator to explicitly clear state to prevent merged corruption).

## Next Steps
Proceed to the Specification phase (`T-00302`) to map out the explicit contract for the recovery and validation interfaces.
