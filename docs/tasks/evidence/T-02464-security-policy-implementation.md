# T-02464: Sandbox Enforcement Security Policy Implementation

## 1. Implementation Overview
This task implements `SandboxSecurityPolicy` and integrates it into `SandboxService` as a pre-execution authorization gate in `code/aiosh-rust/aiosh-core/src/sandbox_service.rs`.

---

## 2. Implementation Details
1. **Security Policy Core (`sandbox_policy.rs`)**:
   - `SandboxPolicyMode`: `Enforcing`, `Permissive`, `Disabled`.
   - `SandboxPolicyVerdict`: `Permit`, `PermitWithWarning`, `Deny`.
   - Invariant enforcement:
     - `SANDBOXPOL1`: Mode-driven gating.
     - `SANDBOXPOL2`: Prohibited command check (e.g. `rm`, `dd`, `mkfs`, `format`, `shutdown`).
     - `SANDBOXPOL3`: Prohibited environment variables (`LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, `PYTHONPATH`, `NODE_OPTIONS`).
     - `SANDBOXPOL4`: Mandatory PEP capability grants for designated sensitive profiles.
     - `SANDBOXPOL5`: Upper-bound ceilings on runtime wall-clock duration and memory.
     - `SANDBOXPOL6`: Bounded JSON serialization and loading (64 KiB cap).
2. **Service Integration (`sandbox_service.rs`)**:
   - Integrated `policy: SandboxSecurityPolicy` into `SandboxService`.
   - Before executing any command, `self.policy.evaluate(request)` is executed.
   - On denial in `Enforcing` mode:
     - Emits an immutable audit record with `outcome: "denied"` and the denial reason into `AuditRing`.
     - Returns `Err("SANDBOXPOL_ERR_DENIED: ...")`.
   - Exposed `policy()`, `policy_mut()`, and `set_policy()` methods on `SandboxService`.

---

## 3. Compilation Verification
`cargo check --workspace` clean with 0 warnings, 0 errors.
