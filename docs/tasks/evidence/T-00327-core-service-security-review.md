# T-00327 — Dependency & Toolchain Pinning: core service Security Review

## 1. Overview
This task conducts a security review of the `toolchain_service` core module and its integration in the `aiosh-cli` frontend (`aiosh toolchain check`). 

## 2. Input Validation & Injection Risks
- **Command Injection**: The `enforce_toolchain` service uses `std::process::Command::new()` with hardcoded binary paths (`rustc`, `python3`, `python`, `node`) and hardcoded arguments (`-V` or `-v`). User input (from the manifest) is never passed into the command vector or the binary name. Shell injection is structurally impossible.
- **Path Traversal (Manifest Source)**: `ToolchainManifest::from_env()` resolves the manifest path from the `AIOSH_TOOLCHAIN_CONFIG` environment variable. While an operator can point this to an arbitrary file, `std::fs::File::open` is restricted by the process's standard permissions, and the parser uses `serde_json` to safely decode the file contents.
- **Denial of Service (DoS)**: The `from_source` manifest reader includes a bounded read (`f.take(65_536).read_to_string(...)`) to prevent memory exhaustion if a malicious actor points the config to `/dev/zero` or a massive file.

## 3. PEP Gating & Audit Logging
- **State Changes**: The `aiosh toolchain check` command is entirely read-only. It inspects the host environment but modifies no system state, resources, or files.
- **PEP Enforcement**: Because the action is read-only and reversible, it does not require explicit PEP grants.
- **Audit Emission**: The `cmd_toolchain` integration strictly emits a `toolchain.check` audit row (via `classify_and_emit` or `emit()`) on *both* the success and error paths. This adheres to ADR-0035 A F-2 (honest audit rows for outcomes).

## 4. Abuse Scenarios & Mitigations
### Scenario A: Spoofing the configuration file
An attacker sets `AIOSH_TOOLCHAIN_CONFIG=/tmp/malicious.json` to bypass checks.
**Mitigation**: The `to_json_with_sources()` function explicitly tags the config as `"source": "env"` when the environment variable is used. This provenance is written to the audit log `args`, leaving an undeniable trail of the override. 

### Scenario B: Spoofing the binaries in PATH
An attacker overrides `PATH` to point to malicious `rustc` or `python3` scripts.
**Mitigation**: The core service runs within the host environment. If the agent/user has already compromised the host's `PATH`, they have already bypassed system boundaries. This toolchain pinning mechanism is designed to catch accidental drift, not defend against an active attacker who controls the `PATH`. 

## 5. Conclusion
No policy bypasses or security vulnerabilities were identified. The module correctly bounds its reads, prevents shell injection by design, and adheres to the AIOS audit log constraints. No blocking notes are required.
