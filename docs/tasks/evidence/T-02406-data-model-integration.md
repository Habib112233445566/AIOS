# T-02406: Sandbox Enforcement Data Model Integration

## 1. Integration Scope
Integrated the Sandbox Enforcement Data Model into the real call paths:
1. `code/aiosh-rust/aiosh-core/src/sandbox.rs`:
   - Added `SandboxPolicy::to_profile(name)` and `SandboxPolicy::from_profile(&SandboxProfile)` conversion bridge.
   - Connected legacy `SandboxPolicy` consumers with typed `SandboxProfile` constructs.
2. `code/aiosh-rust/aiosh-sandbox/src/main.rs`:
   - Updated the binary entrypoint to accept `--profile <name_or_json>`.
   - Supports named stock profiles (`standard`, `strict`, `permissive`) as well as customized serialized JSON profiles.
   - Preserves backward compatibility with `--policy <json>`.
   - Bridges parsed profiles directly to the kernel Landlock and seccomp execution engine.

## 2. Parity & System Linkage
- Preserved byte-level canonical JSON compatibility and hashing semantics.
- Verified workspace builds cleanly with zero errors across `aiosh-core`, `aiosh-sandbox`, `aiosh-cli`, and `aiosh-mcp`.
