# T-01641: Kernel Module Management — Configuration: Research

## Metadata
- **Task ID:** `T-01641`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Kernel Module Management
- **Status:** Complete — research only. No production code changed.
- **Date:** 2026-09-19
- **Dependencies:** `T-01640` (closes Kernel Module Management MCP API surface; this task opens the *configuration* sub-chain).
- **Feeds:** `T-01642` — "configuration: Specification" (`docs/tasks/evidence/T-01642-configuration-specification.md`).
- **Artifacts:** `docs/tasks/evidence/T-01641-configuration-research.md` and `docs/tasks/evidence/T-01641-research.md`.

---

## 1. Scope & Method

The configuration subsystem for Kernel Module Management in AIOS defines how kernel module policies, modprobe directives, autoload profiles, and hardware/security presets are represented, validated, stored, and exported.

### Configuration Surface Breakdown
1. **Canonical Store Schema (`KernelModuleConfig` & `KernelModuleStore`)**:
   - `KernelModuleConfig`: `id: String`, `description: String`, `rules: Vec<ModprobeRule>`, `autoload_modules: Vec<String>`, `created_at: String`.
   - `ModprobeRule`: `Blacklist { module }`, `Options { module, options }`, `Install { module, command }`, `Alias { alias, module }`, `Softdep { module, pre, post }`, `Remove { module, command }`.
2. **Validation Invariants (KM1..KM5, KS2)**:
   - `KM1`: Module name syntax matching `^[a-zA-Z0-9_-]+$`, max length 64 bytes.
   - `KM2`: Parameter syntax matching `^[a-zA-Z0-9_.-]+=[a-zA-Z0-9_.,:-]+$`, with no whitespace or newlines.
   - `KM3`: Mutual exclusion conflict detection between autoloading and blacklisting/disabling.
   - `KM4`: Safe install/remove directives (`/bin/true`, `/bin/false` or validated binary paths).
   - `KM5`: Maximum store document size capped at 10 MiB (`MAX_MODULE_DOC_BYTES`).
3. **Canonical Presets (`KernelModulePreset`)**:
   - `cis_hardened_preset()`: Disables unneeded legacy protocols and filesystems (`cramfs`, `freevxfs`, `jffs2`, `hfs`, `hfsplus`, `squashfs`, `udf`, `dccp`, `sctp`, `rds`, `tipc`).
   - `pentest_wireless_preset()`: Configures autoload and options for wireless injection and auditing drivers (`mac80211`, `cfg80211`, `ath9k_htc`, `rt2800usb`, `rtl8187`, `wireguard`).
   - `container_isolation_preset()`: Enables container networking and virtualization primitives (`overlay`, `br_netfilter`, `veth`, `xt_nat`, `ip_tables`, `ip6_tables`).
4. **Export & Target System Integration**:
   - Emitting `/etc/modprobe.d/aios.conf` containing validated modprobe directives.
   - Emitting `/etc/modules-load.d/aios.conf` containing newline-separated modules for systemd-modules-load.
5. **Runtime Introspection**:
   - Parsing `/proc/modules` formatted lines (`<name> <size> <refcount> <used_by> <state> <address>`).

---

## 2. Facts (Code-Cited & Verified)

- **F1 (Data Model & Structs)**: Defined in `code/aiosh-rust/aiosh-core/src/kernel_module.rs`. All structs derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`.
- **F2 (Pre-Commit Conflict Invariant)**: In `code/aiosh-rust/aiosh-core/src/kernel_module_service.rs`:
  - `add_blacklist` checks `self.config.autoload_modules.iter().any(|m| m == module)`. If true, returns an error.
  - `add_autoload` checks `self.config.rules` for `Blacklist` or `Install { command: ".../bin/true" | ".../bin/false" }`. If true, returns an error.
- **F3 (Atomic Persistence)**: `save_to_path` writes to a sibling tempfile (`.tmp.<pid>.<name>`), calls `sync_all()`, and atomically renames to the target path.
- **F4 (Document Bounding)**: `MAX_MODULE_DOC_BYTES` = 10 MiB is enforced during both `load_from_path` (via file metadata length check) and `save_to_path` (via serialized buffer length check).
- **F5 (Standard Export Formats)**:
  - `export_modprobe_conf()` outputs standard `blacklist <mod>`, `options <mod> <opts>`, and `install <mod> <cmd>` lines.
  - `export_modules_load_conf()` outputs module names separated by `\n`.

---

## 3. Assumptions vs Decisions Needed

### Assumptions
- **A1**: Default modprobe configuration directory is `/etc/modprobe.d/` and autoload directory is `/etc/modules-load.d/` on standard Linux FHS systems.
- **A2**: Canonical presets provide adequate baselines for server hardening, wireless assessment, and container isolation.

### Decisions Needed for Specification (T-01642)
| # | Decision | Context |
|---|---|---|
| D1 | Should configuration support loading from multiple modular files or a single canonical JSON store? | Single canonical JSON store with multi-file export matches `fs_layout` precedent. |
| D2 | How should foreign modprobe directives not represented in `ModprobeRule` be handled during import? | Parse known directives, ignore comments and empty lines, fail with explicit error on unknown directives. |
| D3 | Should default store path fall back to `/etc/aios/kernel_modules.json` or require explicit `--store`? | In CLI/MCP, explicit path or in-memory default maintains test isolation. |

---

## 4. References & Citations
- `modprobe.d(5)` Linux Programmer's Manual (kmod modprobe configuration syntax).
- `modules-load.d(5)` systemd module loading at boot specification.
- CIS Linux Benchmark §1.1.1 (Disable Unused Filesystems & Uncommon Network Protocols).
- UAPI / Linux Kernel `/proc/modules` documentation.
