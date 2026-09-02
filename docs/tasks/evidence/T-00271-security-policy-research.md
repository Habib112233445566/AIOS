# T-00271 — Release Packaging & Backup: Security Policy Research

## Goal
Establish facts, constraints, and prior art for the security policy of Release Packaging & Backup.

## Facts (Derived from Existing Code)
1. **Input Validation**: The configuration loader (`aiosh-core/src/release_config.rs`) already rejects absolute paths and path traversal (`..`).
2. **Denial of Service Limits**: 
   - The config loader enforces a hard limit of 64KB on `release.json` (prevents OOM).
   - The backup zipper enforces a 2GB file size cap and ignores symlinks (prevents recursive zip bombs and disk saturation).
3. **Execution Environment**: OS image generation (`genisoimage` on Linux) and ZIP generation run within standard subprocesses (no special capabilities required like `CAP_SYS_ADMIN` because they just build files locally).
4. **Current Policy Gaps**: The MCP handlers for `aios.release.generate` and `aios.backup.create` need to be hooked up to the `aiosh-core` PEP (Policy Enforcement Point) and Audit Ring.

## Prior Art & Authoritative Sources
- **ADR-0035 \S F-2 (Audit Invariants)**: Consequential actions (actions that mutate state, consume heavy resources, or export data) must write exactly one audit row to the ledger.
- **Principle of Least Privilege**: The `aios.backup.create` tool should require an explicit grant for the backup scope.
- **OWASP Path Traversal (WSTG-INPV-04)**: Relying on path canonicalization is error-prone; the best mitigation is rejecting illegal tokens (`..`, `/`) completely, which we have implemented.

## Decisions Needed
1. What exact string scopes will we use for the PEP grants? (Recommendation: `aios.release.generate` and `aios.backup.create`).
2. Should generating a backup require a manual interactive prompt if triggered autonomously by an agent, or is standard grant possession sufficient? (Recommendation: standard grant possession is sufficient as it is explicitly granted by the user).

## Next Steps
Proceed to the Specification phase to formalize the PEP grants and Audit Event schemas for these tools.
