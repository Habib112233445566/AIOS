# T-00291 — Release Packaging & Backup: Documentation Research

## Goal
Establish facts, constraints, and prior art for formally documenting the Release Packaging & Backup epic.

## Facts (Derived from Current State)
- **Current Documentation**: The root `docs/README.md` currently contains a section dedicated to "Release Packaging & Backup". Over the past epics (Security, Automations, Observability), we have appended paragraphs detailing configuration files, PEP gating, and error interception.
- **Audience**: There are two distinct audiences for this feature.
  1. *Human Operators*: Needing to know how to trigger backups before manual intervention, or how to invoke `aiosh release generate` via the CLI.
  2. *Autonomous Agents*: Needing to know how to format MCP payload arguments for `aios.release.generate` and `aios.backup.create`, and how to interpret the `AuditRing` responses.
- **Constraints**: The `docs/README.md` is growing quite large. The AIOS guidelines suggest keeping documentation modular or well-sectioned.

## Decisions Needed
1. **Location**: Should the documentation remain entirely in `docs/README.md`, or should we extract a dedicated `docs/release-and-backup.md` to prevent main-page bloat? (Recommendation: Keep it in `docs/README.md` for Phase 0 simplicity, but reorganize the section for clarity).
2. **Examples**: The current examples use `aiosh release generate`. Should we also include an explicit JSON payload example for the MCP `aios.backup.create` tool? (Recommendation: Yes, providing raw JSON examples ensures agents can confidently construct the `args`).

## Next Steps
Proceed to the Specification phase (`T-00292`) to map out the exact structure and text for the consolidated documentation updates.
