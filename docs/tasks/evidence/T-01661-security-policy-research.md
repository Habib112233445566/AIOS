# T-01661: Security Policy Research

## Sub-Epic
Kernel Module Management / Security Policy

## Objective
Research existing implementations, prior art, standards, and constraints for the Security Policy of Kernel Module Management in AIOS.

## 1. Problem Statement & Threat Model
Kernel modules execute in Ring 0 (kernel space) with unrestricted supervisor privileges. Unauthorized loading, improper configuration, or tampering with kernel modules introduces critical vulnerabilities:
1. **Malicious or Vulnerable Drivers**: Attackers can load deprecated or vulnerable kernel drivers (e.g. legacy filesystems such as `cramfs`, `jffs2`, `freevxfs`, or network protocols such as `dccp`, `sctp`, `rds`, `tipc`) to trigger kernel privilege escalation.
2. **Arbitrary Command Injection in `install` Directives**: Modprobe `install` directives execute shell commands upon module load. Unrestricted commands allow local code execution under root.
3. **Parameter Injection**: Unsanitized module options may pass malformed arguments to kernel drivers or induce kernel panics/exploits.
4. **Denial of Service via Critical Module Blacklisting**: Blacklisting core filesystem drivers (e.g., `overlay`, `ext4`) or device-mapper modules (`dm_mod`, `crypto`) can render the host or rootfs unbootable.
5. **Unauthorized Agent Mutations**: In an agentic environment, autonomous subagents must not be able to arbitrarily modify kernel configuration without verifiable PEP (Policy Enforcement Point) authorization.

## 2. Standards & Prior Art
- **CIS Linux Benchmark (v3.0)**:
  - Section 1.1.1.x: Ensure mounting of `cramfs`, `freevxfs`, `jffs2`, `hfs`, `hfsplus`, `squashfs`, `udf`, `vfat` is disabled or controlled via blacklist/install `/bin/true`.
  - Section 3.4.x: Ensure uncommon network protocols (`dccp`, `sctp`, `rds`, `tipc`) are disabled.
  - Section 3.5.x: Ensure rare hardware protocols (`firewire-core`, `thunderbolt`) are restricted.
- **Linux Kernel Security Architecture**:
  - `CONFIG_MODULE_SIG_FORCE` and `module.sig_enforce=1`: Cryptographic signature verification.
  - Kernel Lockdown LSM (`lockdown=integrity`, `lockdown=confidentiality`): Blocks unsigned module loading, raw memory access (`/dev/mem`, `/dev/kmem`), and MSR tampering.
- **AIOS Policy Subsystem Precedents**:
  - `aiosh-core/src/service_policy.rs`: Standardized `PolicyMode` (Enforcing, Audit, Permissive), `PolicyViolation`, `PolicyVerdict`, `MAX_POLICY_FILE_BYTES` (64 KiB).
  - `aiosh-core/src/package_policy.rs` & `base_image_policy.rs`: Invariant checks SP1..SP6, deterministic validation, JSON serialization.

## 3. PEP Architecture & Grant Scopes
The Kernel Module Management PEP mandates granular authorization scopes:
- `aios.kernel_module.read`: Query and inspect active modules, parameters, blacklist, and autoload lists.
- `aios.kernel_module.blacklist`: Add or remove blacklist and disable directives.
- `aios.kernel_module.autoload`: Add or remove modules from `/etc/modules-load.d/`.
- `aios.kernel_module.options`: Configure module parameters in `/etc/modprobe.d/`.
- `aios.kernel_module.preset`: Apply pre-packaged security hardening baselines (e.g., `cis_hardened`).
- `aios.kernel_module.export` / `import`: Ingest or export modprobe configuration files.

## 4. Proposed Invariants (SP-KM1 .. SP-KM6)
- **SP-KM1 (Identifier & Parameter Hygiene)**: Module names must match `^[a-zA-Z0-9_]+$`, length $\le 64$. Parameter keys must match `^[a-zA-Z0-9_]+$`, length $\le 128$. Parameter values must be $\le 1024$ characters, with no control characters or shell metacharacters (`;`, `&`, `|`, `` ` ``, `$`, `\n`, `\r`, `\0`).
- **SP-KM2 (Mandatory Blacklist Enforcement)**: Prohibited modules defined in the policy (e.g. legacy filesystems, obsolete network protocols) must never be permitted in autoload or options lists.
- **SP-KM3 (Protected / Immutable Module Guard)**: Essential kernel infrastructure modules (`ext4`, `xfs`, `overlay`, `crypto`, `dm_mod`, `vfat` if EFI) cannot be blacklisted or disabled, preventing operator/agent self-DoS.
- **SP-KM4 (Install Command Sanitization)**: `install` directives may only map to approved benign binaries (`/bin/true`, `/bin/false`, `/usr/bin/true`, `/usr/bin/false`). Arbitrary shell execution is forbidden.
- **SP-KM5 (Privilege & Capability Boundaries)**: Requests must carry verified PEP credentials satisfying the required grant scope.
- **SP-KM6 (Tri-State Policy Evaluation & Bounded Config)**: Policies must support `Enforcing`, `Audit`, and `Permissive` modes, and policy files must not exceed 64 KiB (`MAX_POLICY_FILE_BYTES`).

## 5. Reusable Components & Structure
- Module location: `code/aiosh-rust/aiosh-core/src/kernel_module_policy.rs`.
- Registration in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Structs:
  - `KernelModulePolicyMode` (`Enforcing`, `Audit`, `Permissive`)
  - `KernelModuleSecurityPolicy`
  - `KernelModulePolicyViolation`
  - `KernelModulePolicyVerdict`
- Unit tests in `code/aiosh-rust/aiosh-core/tests/test_kernel_module_policy.rs`.
- Integration tests in CLI and MCP surfaces.
