# T-00309 — Release Packaging & Backup: Recovery & Validation Documentation

## Overview
We updated `docs/README.md` to comprehensively document the new recovery and validation functionality, making it discoverable for human operators and AI agents interacting with the system over MCP.

## Documentation Additions
1. **Usage Examples**: Added a copy-pasteable JSON MCP payload for invoking `aios.backup.restore`.
2. **Security Posture**: Expanded the PEP Gating section to explicitly state that `aios.backup.restore` is an irreversible tool requiring a grant, while `validate_release` and `validate_backup` are read-only and bypass the gate.
3. **Hardening Notes**: Explicitly documented the zip-bomb protections (100k file cap, 10 GB uncompressed size cap) and the Zip-Slip path traversal guard.
4. **Testing Context**: Explained how configuration boundaries, logic, and recovery processes are mapped under `cargo test -p aiosh-core release`.

## Validation
- `docs/README.md` formatting is clean and the new limits are honestly stated.
- All evidence chains logically append to the historical epic structure.
- The task is structurally complete.
