# T-01001 — Distro Selection & Justification / Data Model: Research

## 1. Prior Art & Target Analysis for AIOS Linux Base

AIOS requires a rock-solid, reproducible, lightweight Linux base operating environment capable of running the `aiosh` Rust agent shell, Python data/agent SDKs, SQLite WAL audit database, and container/VM sandboxes.

### Evaluated Distributions

| Distribution | Init System | C Library | Package Mgr | Kernel Baseline | Suitability for AIOS |
|---|---|---|---|---|---|
| **Debian 12 (Bookworm)** | `systemd` | `glibc` | `apt` / `dpkg` | >= 6.1 LTS | **Primary / Recommended**: Gold standard for Rust + Python binary wheel compatibility, systemd cgroup v2 support, long-term security updates. |
| **Alpine 3.19+** | `OpenRC` / none | `musl` | `apk` | >= 6.1 LTS | **Secondary / Container Base**: Extremely compact (<10MB rootfs), ideal for ephemeral sandbox containers. |
| **Arch / Custom Minimal** | `systemd` | `glibc` | `pacman` | Rolling | Reference for minimal custom rootfs builds. |

## 2. Core Data Structures to Model

1. **`DistroFamily`**: `Debian`, `Alpine`, `Arch`, `CustomMinimal`.
2. **`InitSystem`**: `Systemd`, `OpenRC`, `None`.
3. **`ArchTarget`**: `X86_64`, `Aarch64`.
4. **`CLibrary`**: `Glibc`, `Musl`.
5. **`DistroProfile`**: Container specifying `id`, `name`, `family`, `version`, `init_system`, `arch`, `c_lib`, `min_kernel_version`, `default_packages`, `recommended`.
6. **`DistroEvaluation`**: Score structure rating compatibility, container size, package availability, and security update frequency.

## 3. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| glibc compatibility | Fact | Python wheels with native C/Rust extensions (e.g. numpy, pyarrow) require `glibc` or `musllinux` wheels. Debian `glibc` provides 100% prebuilt wheel compatibility. |
| cgroups v2 & systemd | Fact | Systemd natively manages cgroup v2 hierarchy required for fine-grained AI agent CPU/memory resource quotas. |
| In-tree location | Fact | New module `code/aiosh-rust/aiosh-core/src/distro.rs` registered in `aiosh-core/src/lib.rs`. |
