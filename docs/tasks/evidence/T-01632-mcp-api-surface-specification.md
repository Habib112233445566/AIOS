# Task Completion Evidence: T-01632

## Task Overview
- **Task ID**: T-01632
- **Task Name**: Phase 1 — Linux Base System & Bootable Target / Kernel Module Management / MCP API surface: Specification
- **Sub-Epic**: Sub-Epic 4: Kernel Module Management MCP API Surface
- **Status**: Completed

## Specification Details
Formally specified the 10 MCP tools for Kernel Module Management (`aios.kernel_module.*`) in `code/aiosh-rust/aiosh-mcp/src/main.rs`.

### 1. Tool Schemas & Parameters

#### `aios.kernel_module.list`
- **Description**: List loaded kernel modules and configured store rules.
- **Input Schema**:
  - `store_path` (string, optional): Path to persistent kernel module store JSON.
  - `proc_modules_path` (string, optional): Path to mock or alternative `/proc/modules`.
  - `grant_id` (string, optional): Optional PEP grant token.
- **Output**: `{ "ok": true, "data": { "loaded_modules": [...], "rules": [...], "autoload_modules": [...] }, "error": null }`.

#### `aios.kernel_module.get`
- **Description**: Inspect a specific kernel module (live status, parameters, dependencies, store rules).
- **Input Schema**:
  - `module` (string, required): Kernel module identifier.
  - `store_path` (string, optional): Path to persistent store JSON.
  - `proc_modules_path` (string, optional): Path to mock or alternative `/proc/modules`.
  - `grant_id` (string, optional): Optional PEP grant token.
- **Output**: `{ "ok": true, "data": { "module": {...}, "rules": [...], "autoload": bool }, "error": null }`.

#### `aios.kernel_module.blacklist`
- **Description**: Add a module to the blacklist in the kernel module store.
- **Input Schema**:
  - `module` (string, required): Module to blacklist.
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "module": "...", "blacklisted": true }, "error": null }`.

#### `aios.kernel_module.unblacklist`
- **Description**: Remove a module from the store blacklist.
- **Input Schema**:
  - `module` (string, required): Module to unblacklist.
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "module": "...", "removed": bool }, "error": null }`.

#### `aios.kernel_module.options`
- **Description**: Set parameter options for a kernel module in the store.
- **Input Schema**:
  - `module` (string, required): Target module.
  - `options` (array of strings, required): List of parameter settings (`key=value` or flag).
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "module": "...", "options": [...] }, "error": null }`.

#### `aios.kernel_module.autoload`
- **Description**: Add a module to the boot autoload list (`/etc/modules-load.d/`).
- **Input Schema**:
  - `module` (string, required): Target module.
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "module": "...", "autoload": true }, "error": null }`.

#### `aios.kernel_module.unautoload`
- **Description**: Remove a module from the autoload list.
- **Input Schema**:
  - `module` (string, required): Target module.
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "module": "...", "removed": bool }, "error": null }`.

#### `aios.kernel_module.preset.list`
- **Description**: List available canonical presets (`cis_hardened_baseline`, `pentest_wireless_baseline`, `container_isolation_baseline`).
- **Input Schema**:
  - `grant_id` (string, optional): Optional PEP grant token.
- **Output**: `{ "ok": true, "data": [ ... presets ... ], "error": null }`.

#### `aios.kernel_module.preset.apply`
- **Description**: Apply a canonical preset into the kernel module store.
- **Input Schema**:
  - `preset_name` (string, required): Name of preset to apply.
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): PEP authorization grant ID.
- **Output**: `{ "ok": true, "data": { "preset": "...", "applied": true }, "error": null }`.

#### `aios.kernel_module.export`
- **Description**: Export generated `modprobe.d` and `modules-load.d` configuration strings.
- **Input Schema**:
  - `store_path` (string, optional): Path to store JSON.
  - `grant_id` (string, optional): Optional PEP grant token.
- **Output**: `{ "ok": true, "data": { "modprobe_conf": "...", "modules_load_conf": "..." }, "error": null }`.

### 2. Invariants (KM-M1..KM-M5)
- **KM-M1: PEP Capability Gating**: Mutation tools check for authorized capabilities or valid grants.
- **KM-M2: Path Bounds & Validation**: `store_path` and `proc_modules_path` bounded to 1024 bytes and checked for control characters.
- **KM-M3: Uniform Envelope**: Returns standard `{ "ok": bool, "data": ..., "error": ... }`.
- **KM-M4: Store Coherence**: Directly shares `KernelModuleStore` with the CLI surface.
- **KM-M5: Audit Provenance**: Emits audit records into the SQLite WAL audit ring with classifier provenance.
