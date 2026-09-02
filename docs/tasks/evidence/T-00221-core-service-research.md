# T-00221 — Phase 0 — Release Packaging & Backup / Core Service: Research

## Goal
Establish facts, constraints, and prior art for the core service of Release Packaging & Backup (moving from data-model to actual file IO).

## Facts vs. Assumptions

1. **ISO Generation (Releases)**
   - *Fact*: Building a bootable ISO 9660 / El Torito image from scratch in pure Rust is extremely complex and error-prone. Standard practice is to invoke established system utilities like `xorriso`, `genisoimage`, or `mkisofs`.
   - *Assumption*: We will use `std::process::Command` in `aiosh-core` to invoke `genisoimage` (or `xorriso`) to assemble the ISO based on the `target_os` and `components` passed via the data model. 
   - *Constraint*: AIOS targets Linux-based environments (or WSL), where these tools are typically available. A graceful fallback or informative error must be emitted if the binary is not on `PATH`.

2. **ZIP Archiving (Backups)**
   - *Fact*: The data model currently generates a virtual backup path (`aios_backup_{timestamp}.zip`). Creating zip archives from folders can be done either by spawning `zip`/`tar` or via a native Rust crate (`zip` crate).
   - *Assumption*: Adding the `zip` crate to the workspace `Cargo.toml` is the cleanest and most cross-platform approach for Rust, avoiding dependency on the system `zip` binary and offering better error handling and bounded I/O sizes.
   - *Constraint*: Memory-bound file writing is critical; streaming the ZIP creation directly to disk prevents Out-Of-Memory (OOM) faults on large directories.

3. **Audit and Parity (ADR-0035)**
   - *Fact*: The core logic must not violate the single-row audit emission. The file IO logic must update the `target` and `outcome_detail` in the *same* audit row that was established in the data model.
   - *Fact*: If the external `genisoimage` times out or fails, or if a ZIP encounters permission denied, the failure must be caught and logged cleanly into the audit row as `outcome="error"`.

## Unknowns and Decisions Needed (for Implementation Epic T-222+)

1. **Dependency Approval**: Do we approve the addition of the `zip` crate to `workspace.dependencies`, or should we spawn `tar -czf` / `zip` instead? (Recommendation: `zip` crate for robustness).
2. **ISO Tooling Dependency**: Which ISO binary should we strictly require? (Recommendation: `genisoimage`, as it is highly standard across Debian/Ubuntu).
3. **Caps and Timeouts**: We must enforce a timeout on `genisoimage` execution (e.g., 5 minutes) and a size cap on directory traversal for backups to avoid resource exhaustion.

## Conclusion
The physical layer for Release Packaging and Backup requires bounded file-I/O and external process invocation. We will proceed to the implementation plan by addressing these file-system operations securely.
