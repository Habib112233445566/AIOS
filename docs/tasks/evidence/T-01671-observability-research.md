# T-01671: Observability Research

## Sub-Epic
Kernel Module Management / Observability

## Objective
Establish facts, constraints, and prior art for the Observability subsystem of Kernel Module Management in AIOS (`KernelModuleObservabilityReport`).

## 1. Facts vs Assumptions

### Established Facts (from Upstream Linux & Codebase)
1. **Live Module Telemetry (`/proc/modules` & `/sys/module/`)**:
   - `/proc/modules` reports module name, size in bytes, reference count, dependent modules list, state (`Live`, `Loading`, `Unloading`), and memory offset.
   - `/sys/module/<name>/` exposes parameters, refcount, sections, taint flags, and version metadata.
   - `aiosh-core::kernel_module::ModuleInfo` already parses `/proc/modules` lines into structured records.
2. **Declarative Configuration State (`KernelModuleStore`)**:
   - Stores `ModprobeRule` directives (Blacklist, Alias, Options, Install, Remove, Softdep).
   - Stores `autoload_modules` (modules intended to load at boot via `/etc/modules-load.d/`).
3. **Security Policy State (`KernelModuleSecurityPolicy`)**:
   - Provides evaluation rules for prohibited modules, protected modules, install command hygiene, and parameter safety.
4. **AIOS Observability Precedents**:
   - `aiosh-core/src/service_observability.rs`: Standardized report pattern with categorical distributions, health metrics, policy compliance, and deterministic serialization.

### Assumptions to Validate
- Live modules may or may not be present in the persistent store; observability must report both live runtime telemetry and store configuration telemetry.
- In headless/containerized/mock environments, `/proc/modules` may be absent or emulated; report generator must handle missing procfs gracefully using fallback data.

## 2. Telemetry Requirements & Invariants (KO1..KO6)
- **KO1 (Inventory & Aggregation)**: Accurately report count of live modules, store rules, and autoload modules.
- **KO2 (Categorical Distributions)**: Breakdowns by module runtime state (`live`, `loading`, `unloading`) and modprobe rule type (`blacklist`, `alias`, `options`, `install`, `remove`, `softdep`).
- **KO3 (Memory & Footprint Telemetry)**: Aggregate size in bytes consumed by loaded kernel modules in ring 0.
- **KO4 (Dependency & Reference Distribution)**: Reference count distribution (`0`, `1-2`, `3-5`, `6+`) identifying leaf vs heavily dependent modules.
- **KO5 (Security & Compliance Telemetry)**: Policy compliance metrics, violation counts, and prohibited module detection.
- **KO6 (Deterministic Serialization & Bounds)**: Bounded JSON outputs with predictable key ordering for audit comparisons.

## 3. Decisions & Reusable Components
- Target source: `code/aiosh-rust/aiosh-core/src/kernel_module_observability.rs`.
- Register in `code/aiosh-rust/aiosh-core/src/lib.rs`.
- Reuses `KernelModuleStore`, `KernelModuleService`, `ModuleInfo`, and `KernelModuleSecurityPolicy`.
- Expose via CLI: `aiosh mod status` / `aiosh mod metrics` or `aiosh mod observability`.
- Expose via MCP: `aios.kernel_module.observability`.
