# T-02416: Sandbox Enforcement Core Service Integration

## 1. Integration Scope & Architecture
Integrated the Sandbox Enforcement Core Service (`SandboxService`) into the MCP tool surface and system runtime:
1. **MCP Tool Surface (`code/aiosh-rust/aiosh-mcp/src/main.rs`)**:
   - Registered tool `aios.sandbox.profiles`: Allows agents and external controllers to inspect all registered sandbox containment profiles.
   - Registered tool `aios.sandbox.probe`: Exposes host kernel containment capabilities (Landlock LSM support and ABI version, Seccomp-BPF filtering status, `no_new_privs` support).
   - Routed calls through the standard recorded dispatch pipeline (`dispatch::recorded_call`), guaranteeing PEP evaluation and audit ring capture.
2. **Core Service Cross-Substrate Interop**:
   - Backed by SQLite WAL audit ring, preserving strict SHA-256 hash chaining and canonical JSON formatting.

## 2. Verification
Verified compilation across all workspace crates with zero warnings.
