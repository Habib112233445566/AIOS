# T-01669: Security Policy Documentation

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Document the Kernel Module Management Security Policy (`KernelModuleSecurityPolicy`), invariants SP-KM1..SP-KM6, operator CLI commands (`aiosh mod policy`), and MCP tool (`aios.kernel_module.policy`) in `docs/kernel_module_management.md` §10.

## Deliverables
1. **Section 10 of `docs/kernel_module_management.md`**:
   - Invariants overview table: SP-KM1 through SP-KM6.
   - CLI invocation examples:
     - `aiosh mod policy --json`
     - `aiosh mod policy cramfs --json`
     - `aiosh mod policy --evaluate-store --store /etc/aios/kernel_modules.json --json`
     - `aiosh mod policy --policy /etc/aios/kernel_module_policy.json --evaluate-store`
   - MCP invocation example:
     - `aios.kernel_module.policy` with `{ "module": "cramfs" }`.
   - Constraints and known limitations:
     - Disjointness requirement between prohibited and protected modules.
     - 64 KiB file size ceiling and regular file requirement.
     - Non-bypassable critical fatal violations in permissive mode.
   - Cross-references to task evidence files T-01661 through T-01668.

## Verification
- Verified documentation in `docs/kernel_module_management.md`.
- Artifacts: `docs/tasks/evidence/T-01669-security-policy-documentation.md` and `T-01669-documentation.md`.
