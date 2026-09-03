# AIOS Base Linux System: Distribution Selection & Architectural Justification

## 1. Executive Summary & Production Objectives
Phase 1 of AIOS requires a minimal, secure, reproducible, and highly compatible Linux base system. The target runtime hosts autonomous AI agents, local LLM execution runtimes (llama.cpp, ONNX Runtime, PyTorch), kernel-level sandbox isolation (Landlock, seccomp, eBPF), and policy enforcement proxies (PEP).

After empirical evaluation across candidate distributions, **Debian 12 Minimal ("Bookworm") x86_64** was selected as the reference tier-1 distribution profile (`debian-12-minimal-x86_64`).

---

## 2. Multi-Factor Evaluation Model & Scoring Weights
Distributions are scored via a deterministic weighted evaluation function:

$$\text{Score} = (W_{bin} \times S_{bin}) + (W_{sec} \times S_{sec}) + (W_{foot} \times S_{foot})$$

### Default Evaluation Weights (`config/distro.json`)
| Dimension | Weight | Target Attributes |
|---|---|---|
| **Binary Compatibility ($W_{bin}$)** | 0.35 | `glibc` standard compliance, prebuilt Python wheels, PyTorch/CUDA acceleration |
| **Security Posture ($W_{sec}$)** | 0.35 | Signed package repositories, CVE turnaround, default AppArmor/seccomp, minimal attack surface |
| **Footprint Efficiency ($W_{foot}$)** | 0.30 | Rootfs image size $\le 1\text{ GB}$, memory usage $\le 256\text{ MB}$, rapid boot time $\le 3\text{ s}$ |

A distribution profile must achieve $\text{Score} \ge 0.70$ across all dimensions to be certified as **Production Ready**.

---

## 3. Production Reference Profile: Debian 12 Minimal
**Profile ID:** `debian-12-minimal-x86_64`

### Architectural Rationale & Justification
1. **Unrivaled Binary Compatibility ($S_{bin} = 0.95$)**:
   - Standard `glibc` (version 2.36) ensures compatibility with precompiled C/C++ libraries, Rust toolchains, and AI frameworks without requiring complex musl-libc patching or compilation from source.
   - Broadest support for hardware acceleration drivers (NVIDIA CUDA, Intel oneAPI, AMD ROCm).
2. **Security & Supply Chain Rigor ($S_{sec} = 0.90$)**:
   - Strong cryptographic package signing via `apt` with Debian security team updates.
   - Kernel support for AppArmor, Landlock LSM, and seccomp BPF out of the box.
   - 5-year Long Term Support (LTS) lifecycle ensures deterministic stability.
3. **Optimized Minimal Footprint ($S_{foot} = 0.85$)**:
   - Stripped minimal rootfs installation ($\approx 480\text{ MB}$) avoids desktop environments, unnecessary daemons, or bloatware.
   - Clean integration with systemd unit sandboxing and cgroups v2 resource quotas.

---

## 4. Alternative Profiles & Comparative Trade-Off Matrix

| Metric | Debian 12 Minimal | Alpine Linux 3.19 | Arch Linux | Fedora CoreOS |
|---|---|---|---|---|
| **Family** | Debian | Alpine | Arch | RedHat / Fedora |
| **C Standard Library** | `glibc` 2.36 | `musl` 1.2.4 | `glibc` 2.39 | `glibc` 2.38 |
| **Package Manager** | `apt` | `apk` | `pacman` | `rpm-ostree` |
| **Rootfs Size** | ~480 MB | ~12 MB | ~800 MB | ~1.8 GB |
| **Security Score** | 0.90 | 0.90 | 0.65 | 0.88 |
| **Binary Compat Score** | 0.95 | 0.65 | 0.90 | 0.90 |
| **Footprint Score** | 0.85 | 0.95 | 0.60 | 0.55 |
| **Overall Score** | **0.90** | 0.83 | 0.72 | 0.79 |
| **Production Ready** | **YES** | NO (musl compat floor) | NO (rolling release) | CONDITIONAL |

### Trade-Off Findings
- **Alpine 3.19**: While boasting exceptional footprint ($S_{foot} = 0.95$), its `musl` libc fails the AIOS binary compatibility floor ($S_{bin} = 0.65 < 0.70$), causing catastrophic failures when loading pre-built wheels and shared libraries. Alpine remains designated for lightweight container tasks.
- **Arch Linux**: Excellent upstream freshness, but its rolling-release model introduces non-deterministic package updates that violate AIOS immutability and reproducibility goals.
- **Fedora CoreOS**: Strong immutability via `rpm-ostree`, but higher base memory and disk footprint make it sub-optimal for minimal edge/virtualized deployments.

---

## 5. Subsystem Architecture

### 5.1 Data Model (`aiosh-core::distro`)
- `DistroProfile`: Core specification containing `id`, `name`, `version`, `family`, `arch`, `kernel_version`, `libc`, `init_system`, `package_manager`, and `is_immutable`.
- `DistroEvaluation`: Multi-attribute evaluation record tracking `overall_score`, dimension scores, and boolean `is_production_ready`.
- `DistroStore`: In-memory registry with transactional SQLite WAL audit persistence and JSON serialization.

### 5.2 Configuration Subsystem (`aiosh-core::distro_config`)
- Canonical file: `config/distro.json`.
- Environment overrides:
  - `AIOSH_DISTRO_CONFIG`: Custom configuration file path.
  - `AIOSH_DISTRO_STORE_PATH`: Custom store file path.
  - `AIOSH_DEFAULT_DISTRO`: Fallback profile ID.
- Security constraints: 64 KiB configuration file size cap, path traversal (`..`) rejection, and IEEE 754 `NaN` rejection.

### 5.3 Security Policy Subsystem (`aiosh-core::distro_policy`)
- Enforcement rules:
  - Minimum security score floor: $\ge 0.70$.
  - Minimum binary compatibility score floor: $\ge 0.70$.
  - Repository HTTPS encryption requirement.
  - Signed package requirement.
  - Disallowed family blacklisting (e.g. `AIOSH_DISTRO_DISALLOWED_FAMILIES`).

### 5.4 Observability Subsystem (`aiosh-core::distro_observability`)
- Telemetry contract: `DistroObservabilityReport`.
- Invariant validations:
  - **O1**: $\sum \text{family counts} = \text{total profiles}$.
  - **O2**: $\sum \text{architecture counts} = \text{total profiles}$.
  - **O3**: $\text{production ready count} \le \text{total profiles}$, $\text{policy compliant count} \le \text{total profiles}$.
  - **O4**: All score averages are clamped strictly within $[0.0, 1.0]$.

---

## 6. Command Reference

### CLI Surface (`aiosh distro`)
```bash
# List all registered distro profiles
aiosh distro list [--json] [--store <path>]

# Show details for a specific distro profile
aiosh distro show debian-12-minimal-x86_64 [--json]

# Evaluate profiles against AIOS evaluation criteria
aiosh distro evaluate [debian-12-minimal-x86_64] [--json]

# Query the recommended reference profile
aiosh distro recommend [--json]

# Inspect configuration provenance and active weights
aiosh distro config [--json]

# Audit profiles against AIOS security policy standards
aiosh distro policy [debian-12-minimal-x86_64] [--json]

# Retrieve system-wide observability and telemetry metrics
aiosh distro stats [--json]
```

### MCP Tool Surface
| Tool Name | Description | Key Arguments |
|---|---|---|
| `aios.distro.list` | List registered Linux distribution profiles | `store_path`, `grant_id` |
| `aios.distro.show` | Get detailed distro profile specification | `id`, `store_path`, `grant_id` |
| `aios.distro.evaluate` | Multi-criteria evaluation of profiles | `id`, `store_path`, `grant_id` |
| `aios.distro.recommend` | Return designated production distro | `store_path`, `grant_id` |
| `aios.distro.policy` | Audit compliance against security policy | `id`, `store_path`, `grant_id` |
| `aios.distro.stats` | Retrieve aggregated observability telemetry | `store_path`, `grant_id` |
