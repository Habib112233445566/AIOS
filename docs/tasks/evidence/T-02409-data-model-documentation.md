# T-02409: Sandbox Enforcement Data Model Documentation

## 1. Overview
The **Sandbox Enforcement Data Model** provides structured, type-safe security profiles and execution specifications for isolated command execution in AIOS userspace. It models:
- **Sandbox Profiles**: `Strict`, `Standard`, `Permissive`, `IsolatedDev`, and `Custom`.
- **Resource Constraints**: Strict limits on memory allocation, CPU time, wall-clock time, process spawn count, and open file descriptors.
- **Filesystem Permissions**: Separation into read-only, read-write, executable, and denied paths, with built-in path-traversal rejection.
- **Syscall Filters**: 17 dangerous default syscalls with custom allow/denylist additions.
- **Network Modes**: `Disabled`, `LoopbackOnly`, `FilteredEgress`, `Unrestricted`.
- **Environment Policy**: Host environment stripping (`clean_env`), pass-through allowlist, and sanitized injected variables.

## 2. API & Usage Examples

### Rust Code: Constructing a Custom Sandbox Profile
```rust
use aiosh_core::sandbox_data_model::{
    IsolationLevel, NetworkIsolationMode, SandboxProfileBuilder, SandboxProfileType,
};

let profile = SandboxProfileBuilder::new("agent-data-worker")
    .profile_type(SandboxProfileType::Custom)
    .isolation_level(IsolationLevel::FullLandlockSeccomp)
    .max_memory_bytes(1_073_741_824) // 1 GB
    .max_wall_time_ms(15_000)        // 15 s
    .add_ro_path("/usr")
    .add_ro_path("/lib")
    .add_rw_path("/tmp/workspace")
    .add_execute_path("/usr/bin/python3")
    .add_denied_path("/etc/shadow")
    .network_mode(NetworkIsolationMode::LoopbackOnly)
    .add_denied_syscall("ptrace")
    .set_env_var("AIOS_SANDBOX", "1")
    .build()?;
```

### CLI Invocation via `aiosh-sandbox`
```bash
# Using a stock profile name:
aiosh-sandbox --profile standard -- /bin/ls -la /tmp

# Using a strict profile with no network and clean environment:
aiosh-sandbox --profile strict -- /usr/bin/whoami

# Passing serialized custom profile JSON:
aiosh-sandbox --profile '{"name":"custom","profile_type":"custom","isolation_level":"full_landlock_seccomp","resources":{"max_memory_bytes":536870912,"max_cpu_time_ms":10000,"max_wall_time_ms":30000,"max_processes":32,"max_open_files":256},"filesystem":{"paths_ro":["/usr","/etc"],"paths_rw":["/tmp"],"paths_execute":["/usr/bin"],"paths_denied":[],"allow_cwd_rw":true,"allow_tmp_rw":true},"network":"disabled","syscall":{"no_new_privs":true,"default_action":"allow","denylist":["ptrace","mount"],"allowlist":[]},"environment":{"clean_env":true,"allow_vars":["PATH"],"injected_vars":{}}}' -- /usr/bin/env
```

## 3. Constraints & Known Limitations
1. **Linux Kernel Dependency for Native Isolation**:
   - Landlock filesystem filtering requires Linux kernel $\ge$ 5.13.
   - Seccomp-BPF requires kernel support and x86_64 architecture for numeric syscall tables.
   - On non-Linux systems (Windows, macOS) or unprivileged containers, the runtime honestly reports unavailable components in the execution result envelope (`FAIL: unsupported on non-Linux`) while executing with fallback controls.
2. **Path Traversal Invariant**:
   - All paths must be normalized and cannot contain `..`.
3. **Upper Caps**:
   - Max 256 paths per policy, max path length 4096 bytes, max wall-clock time 1 hour, max memory 64 GB.
