# T-01521: Filesystem Layout - CLI Surface: Research

## Metadata
- **Task ID:** `T-01521`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Filesystem Layout CLI Surface (`code/aiosh-rust/aiosh-cli::cmd_fs_layout`)
- **Status:** Complete
- **Date:** 2026-09-16
- **Milestone:** Sub-Epic: Filesystem Layout (1/10) — CLI Surface Research
- **Dependencies:** `T-01520` (Core Service Verification & Evidence)
- **Next Task:** `T-01522` (Filesystem Layout / CLI surface: Specification)

---

## 1. Executive Summary & Problem Context

With the Filesystem Layout data model (`T-01501..T-01510`) and core service coordinator (`T-01511..T-01520`) fully verified, Task `T-01521` initiates the **CLI Surface Sub-Epic** (`T-01521..T-01530`). 

The operator command-line interface (`aiosh layout` and alias `aiosh fs-layout`) serves as the primary terminal tool for systems administrators, Kali penetration testers, and deployment automation scripts to:
1. Inspect, list, and register filesystem layout specifications.
2. Probe target storage block devices for capacity feasibility prior to partitioning.
3. Compare layout deltas and identify destructive modifications before applying changes.
4. Export and import Linux `/etc/fstab` configuration files.
5. Manage the active layout pointer and persistent store state on disk.

---

## 2. Upstream Standards & CLI Prior Art

1. **Linux Storage Utilities (`util-linux`)**:
   - `lsblk(8)`: Tabular output displaying block device names, sizes, partition types, and mount points.
   - `fdisk(8)` / `sfdisk(8)`: GPT partition table dumps displaying partition indices, start/end sectors, types, and labels.
   - `findmnt(8)`: Tree and tabular hierarchy displaying active mount topologies, filesystem types, and mount flags.
   - *Design Implication*: `aiosh layout show` and `aiosh layout list` should mimic `lsblk` and `findmnt` clarity in human-readable output, while providing standardized JSON for automated parsing.

2. **POSIX Utility Conventions & Exit Codes**:
   - Exit Code `0`: Clean execution / operation succeeded / valid specification.
   - Exit Code `1`: Operational failure, invariant violation, target capacity deficit, or validation failure.
   - Exit Code `2`: Command line usage error, missing mandatory argument, unknown subcommand, or invalid flag syntax.
   - *Design Implication*: The CLI command must strictly obey this exit code convention across all subcommands.

3. **Standard Result Envelopes**:
   - When `--json` is supplied, stdout must contain a single canonical JSON object:
     ```json
     {
       "code": 0,
       "data": { ... },
       "error": null
     }
     ```
     or on error:
     ```json
     {
       "code": 1,
       "data": null,
       "error": {
         "code": "VALIDATION_FAILED",
         "message": "FL1 violation: ..."
       }
     }
     ```

4. **Existing AIOS CLI Subsystems Prior Art**:
   - Subsystems such as `aiosh session`, `aiosh package`, and `aiosh service` support a standardized command pattern:
     - Inspection: `list`, `show <id>`, `config`.
     - Evaluation: `validate`, `check`, `policy`, `stats`.
     - Operations: `probe`, `diff`, `action` / `set-active`.
     - Store isolation: `--store <path>` allowing testing on ephemeral files without mutating `/etc/aios/`.

---

## 3. Facts vs. Assumptions

| Domain | Verified Fact | Technical Assumption |
|---|---|---|
| **Invocation Routing** | `aiosh-cli/src/main.rs` dispatches commands based on the first argument (`args[0]`). | Both `aiosh layout` and `aiosh fs-layout` can be routed identically to `cmd_fs_layout`. |
| **Output Modes** | Operators require clear terminal text, while CI pipelines and AI agents require machine-readable JSON. | The presence of `--json` anywhere in `rest` args should toggle output format across all subcommands. |
| **Store Path** | Production stores default to `/etc/aios/fs_layouts.json`. | Operators running in unprivileged or testing environments need `--store <path>` to target local sandbox JSON files. |
| **Active Pointer** | The store tracks exactly one `active_layout_id`. | CLI commands like `show`, `validate`, `probe`, and `fstab` should operate on the active layout by default if no `--spec`, `--container`, or ID is passed. |
| **Destructive Awareness** | Re-partitioning or re-formatting filesystems destroys existing data. | `aiosh layout diff` should clearly highlight whether a transition between two layouts is destructive (`destructive: true`). |
| **Audit Logging** | Every operator CLI invocation must be logged to the append-only SQLite WAL ring. | `classify_and_emit` must be called on every path (success and failure) recording actor, command, target, and outcome. |

---

## 4. Planned Subcommands & Flags for `aiosh layout`

```
aiosh layout [SUBCOMMAND] [OPTIONS]

SUBCOMMANDS:
  list                 List all registered filesystem layout profiles in the store
  show [ID]            Display detailed configuration of a layout (default: active layout)
  validate             Validate layout specification against FL1..FL5 invariants
  probe                Evaluate target disk capacity and feasibility against a layout
  diff [SRC] [TGT]     Compute differential comparison between two layouts
  fstab                Generate standard 6-field /etc/fstab file content
  register --spec <F>  Register a new layout specification into the store
  set-active <ID>      Switch the active layout pointer
  remove <ID>          Remove a non-active custom layout from the store
  import-fstab <F>     Import external fstab into a new layout specification

GLOBAL OPTIONS:
  --json               Emit output formatted as structured JSON
  --standard           Select canonical UEFI standard reference preset
  --container          Select minimal container reference preset
  --spec <file_or_str> Provide custom layout specification via file or JSON string
  --store <path>       Target custom layout store file path (default: /etc/aios/fs_layouts.json)
  --bytes <N>          Specify target block device size in bytes for probing
  --help, -h           Display help and usage information
```

---

## 5. Unknowns and Decisions Needed

1. **Decision 1: Direct Mutating Commands in CLI**:
   - *Question*: Should the CLI support `register`, `set-active`, and `remove`?
   - *Resolution*: Yes. These mutations manipulate declarative metadata in the layout store file (`/etc/aios/fs_layouts.json` or `--store <path>`), not physical disk sectors. They do not format drives.
2. **Decision 2: Handling Missing Store File**:
   - *Question*: If `/etc/aios/fs_layouts.json` does not exist on disk when `aiosh layout list` is called, what should happen?
   - *Resolution*: Initialize an in-memory store pre-seeded with built-in presets (`standard_uefi`, `minimal_container`), rather than failing with file not found.
3. **Decision 3: Ingestion Ceiling**:
   - *Question*: What maximum file size should `--spec` and `--fstab` accept?
   - *Resolution*: Enforce a strict 10 MiB limit on all file reading, with path traversal and control character sanitization.

---

## 6. Citations & Bibliography

1. IEEE Std 1003.1-2017 (POSIX.1) — Utility Conventions and Exit Statuses.
2. Kerrisk, Michael. *The Linux Programming Interface*, No Starch Press, 2010.
3. `util-linux` Documentation: `lsblk`, `findmnt`, `fdisk`, `sfdisk`.
4. AIOS Task Architecture & Invariant Enforcement Guidelines (ADR-0035).
