# Documentation Index — v2 (2026-08-20 course correction)

> **v2.1 amendment (2026-08-21) — implementation language.** The shipping
> stack is now **Rust** (`code/aiosh-rust/`: aiosh-core + aiosh-cli +
> aiosh-mcp). The legacy TypeScript (`code/aiosh-cli`) and Python
> (`code/aiosh-mcp`) implementations are retained in-repo as the
> cross-substrate reference contract — they are NOT the ship path. See
> `../README.md` and `../findings.md` (2026-08-21 entry) for details.
>
> **v2 framing.** The product vision has been restated:
> *"a Linux system for ethical hacking on the inside, a Windows-style desktop
> on the outside, with AI as a first-class S-rank kernel subsystem that
> controls the whole system."* Three pillars drive every decision:
> - **Pillar A — Linux ethical-hacking platform** (foundation)
> - **Pillar B — Windows-like desktop** (user-facing surface)
> - **Pillar C — AI as S-rank first-class kernel subsystem** (control plane)
>
> See `../README.md`, `../mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` (v2.0),
> and `../mostimportanAIfolder/PRODUCT_ROADMAP.md` (v2.0) for the canonical v2
> plan.

## Existing research sub-trees (preserved from v1)

The research volumes below informed the v1 product framing (a from-scratch
RISC-V microkernel as shipping target). In v2 they are preserved as
**research substrate** for our userspace capability/IPC/scheduler designs.
Their findings remain relevant and authoritative; they are no longer the
shipping-path definition.

| Directory | What it contains | v2 role |
|---|---|---|
| `research/` | 13 V1-XX files: operating-system theory, systems theory, computer architecture, security principles, AI theory, threat model, system ABI, memory protection, IPC, AI privilege model, research refresh, AI attack surface, AI kernel-safety primitives | Substrate: informs Pillar A, B, and C capability/IPC/PEP designs |
| `research-architecture/` | 27 AI-Native-OS architecture research files covering kernel, memory, networking, storage, virtualization, AI runtime, developer platform, security, observability, performance, hardware | Substrate: explicit OS-level inputs to all three Pillars |
| `gui-pointer-research.md` | GUI pointer research | Inputs Pillar B (Windows-like desktop) |
| `hardware-notes.md` | Hardware notes | Inputs Pillar A drivers and Pillar B input delivery |
| `research_cursor_lag.md` | Cursor-lag notes | Inputs Pillar B rendering / input |

## v2 critical-path artifacts (authoritative)

| Document | Authority |
|---|---|
| `../README.md` | Product pitch and v2 mission |
| `../mostimportanAIfolder/AI_CONSTITUTION.md` v1.1 | Highest — immutable engineering laws incl. ratified S-rank AI principles **P-1..P-6** |
| `../mostimportanAIfolder/AIOS_MASTER_BLUEPRINT.md` v2.0 | Forward plan |
| `../mostimportanAIfolder/PRODUCT_ROADMAP.md` v2.0 | Phased plan |
| `../mostimportanAIfolder/AI_BOOT_PROTOCOL.md` v1.1 | How agents start work |
| `../mostimportanAIfolder/MASTER_PROJECT_EXECUTION_PROTOCOL.md` v1.1 | How agents execute work |
| `../mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md` v1.1 | How agents research without hallucinating (with v2 citation anchors) |

## v2 citation anchors (refresh monthly)

- Kali tool taxonomy — <https://www.kali.org/tools/>
- Kali Linux distro — <https://en.wikipedia.org/wiki/Kali_Linux>
- Parrot OS — <https://en.wikipedia.org/wiki/Parrot_OS>
- BlackArch — <https://en.wikipedia.org/wiki/BlackArch>
- KDE Plasma — <https://en.wikipedia.org/wiki/KDE_Plasma>
- Xfce — <https://en.wikipedia.org/wiki/Xfce>
- Wayland — <https://en.wikipedia.org/wiki/Wayland_(display_server_protocol)>
- Wine — <https://en.wikipedia.org/wiki/Wine_(software)>
- Proton — <https://en.wikipedia.org/wiki/Proton_(software)>
- MCP introduction — <https://modelcontextprotocol.io/introduction>
- AI agent / OS integration — <https://en.wikipedia.org/wiki/AI_agent>
- Anthropic — <https://en.wikipedia.org/wiki/Anthropic>

## Task ledger (Sprint 3+)

The canonical work queue lives in `tasks/`:

| File | Role |
|---|---|
| `tasks/MASTER_TASK_LEDGER.jsonl` | 10,000 sequential tasks (T-00001..T-10000), each with goal, instructions, acceptance criteria |
| `tasks/MASTER_TASK_LEDGER.md` | Human index: phase map + first 25 tasks in detail |
| `tasks/TASK_STATE.json` | Live pointer: `next_task` is the ONLY task allowed to start |
| `tasks/GOALS.md` | Mission, governing-doc precedence, and the NO-SKIP execution law |
| `tasks/evidence/` | Per-task evidence (`T-NNNNN-*.md`) |

**Rule: agents must read `tasks/TASK_STATE.json` and execute only
`next_task`.** Complete a task with the Rust shipping surface
`aiosh task done <id> --note "…"` (export `AIOSH_TASKS_DIR="$PWD/docs/tasks"`
first — see `SPEC-TASK-LEDGER.md` §7 L2) or the legacy
`python3 tools/complete_task.py <id>` — both refuse out-of-order
completion and advance the pointer by exactly one. Never skip ahead.
Agents may also drive the ledger over MCP via the **`aios.task`** tool
(read-only `status`/`check`/`validate`; mutations require a PEP grant) —
operator reference and copy-pasteable calls: `SPEC-TASK-LEDGER.md` §8.
Integrity drift check: `aiosh task validate` (read-only, report-only —
`SPEC-TASK-LEDGER.md` §9).
Data model, command reference, and known limitations:
**`docs/SPEC-TASK-LEDGER.md`** (T-00019/T-00029).
`../mostimportanAIfolder/TASK_DATABASE.json` is a
NON-authoritative graph-derived reconstruction; do not use it to pick work.

## CI Smoke Orchestration (T-00111..T-00120)

`bash ci/run_all_smokes.sh` delegates to **`tools/ci_run.py`**, which
executes the suite registry `tools/ci_suites.py` — the single source of
suite order (order IS contract; suites share rebuild state, never
parallelize). Additions over the legacy bash runner: per-suite wall-clock
**timeouts** with process-group kill, and an atomic machine-readable run
summary.

```bash
# Run full CI with a custom summary location:
AIOSH_CI_RESULTS=/tmp/ci-summary.json bash ci/run_all_smokes.sh

# Consume the result programmatically:
python3 -c "import json;d=json.load(open('/tmp/ci-summary.json'));\
print(d['all_pass'], [(r['suite'],r['status'],r['duration_ms']) for r in d['results']])"
```

Summary schema (stable additive-only key set):
`{tool, schema_version, started_at, finished_at, total, passed, failed,
all_pass, results:[{suite,index,status,exit_code,duration_ms,started_at,
finished_at,log_path}]}`; status ∈ pass|fail|timeout|error.

Limitations (honest): timeout kills the process GROUP, but a suite that
double-forks can still escape it; log files under /tmp are uncapped on
disk (orchestrator memory exposure is bounded to a 64 KiB tail); the
summary is advisory telemetry — the exit code remains the CI verdict.
Evidence: `tasks/evidence/T-00111-research.md` …
`T-00120-data-model-verification-evidenc.md`.
### CI Summary Service (T-00121..T-00130)

The core service parses and validates the machine-readable summary output of ci_run.py. To align with the v2.1 shipping stack mandate, this service is implemented natively in Rust under iosh-core and exposed via the iosh ci CLI command. It serves as a strict gating mechanism that validates artifact schema, arithmetic coherence, and bounds limits, effectively sealing the CI run.

`ash
# Validate the CI output artifact explicitly (defaults to /tmp/aiosh-ci-results.json)
aiosh ci check

# Display a human-readable run report
aiosh ci show

# List only the failing suites and their log paths
aiosh ci failures
`

Limitations (honest): the read operation implements bounded retries to handle orchestrator lock contention but assumes a final artifact will eventually exist. The JSON payload is read completely into memory (capped at 1MB to prevent OOM) rather than streamed. 
Evidence: 	asks/evidence/T-00121-research.md .. T-00130-verification.md (and intermediate evidence like 	asks/evidence/T-00126-core-service-integration.md and 	asks/evidence/T-00128-core-service-hardening.md).


### Release Packaging & Backup (T-0211 - T-0239)
The final stage of Phase 0 implements the physical `aiosh-mcp` tools and the `aiosh` CLI commands for exporting the system snapshot as a bootable ISO or an auditable ZIP.

**Usage Example (CLI):**
```bash
# Generate an OS release ISO
aiosh release generate --os debian-13 --version 1.0.0 --components aiosh-core,aiosh-cli

# Create a system snapshot ZIP backup
aiosh backup create --target-path /var/aios --include-audit true --include-memory false
```

**Usage Example (MCP):**
```json
// Example: Creating an OS Snapshot Backup via MCP
{
  "name": "aios.backup.create",
  "arguments": {
    "target_path": "/var/aios",
    "include_audit": true,
    "include_memory": false
  }
}
```

**Usage Example (Recovery & Validation via MCP):**
```json
// Example: Restoring an OS Snapshot Backup via MCP
{
  "name": "aios.backup.restore",
  "arguments": {
    "backup_path": "/var/aios_backup.zip",
    "target_dir": "/var/aios_restore",
    "grant_id": "grant-abc-123"
  }
}
```

**Configuration:**
Config is loaded natively in Rust from `$AIOSH_RELEASE_CONFIG` or `config/release.json` (defaults to 2GB `max_file_size_bytes` and `output/release` for `output_dir`).

```bash
# Example: Provide custom configuration
echo '{"max_file_size_bytes": 104857600, "output_dir": "custom_output"}' > /tmp/custom_release.json
export AIOSH_RELEASE_CONFIG=/tmp/custom_release.json
aiosh release generate --os debian-13 --version 1.0.0 --components aiosh-core
```

**Security Policy (PEP Gating):**
Both `aios.release.generate`, `aios.backup.create`, and `aios.backup.restore` are classified as irreversible actions. Agents invoking these tools via MCP *must* possess an active cryptographic grant token for the respective tool scope, or the dispatch gate will synchronously reject the invocation with a 403-equivalent refusal. Validation endpoints (`validate_release`, `validate_backup`) are read-only and bypass this PEP requirement.

**Automated Tests:**
Configuration boundaries (OOM protection, path traversal) and snapshot/ISO generation logic are natively covered in `aiosh-core` unit tests. MCP and CLI layers use Python wrappers to verify physical file behaviors. Recovery logics (Zip-Slip defense, state corruption defense) are explicitly tested in `recovery_tests`.

**Observability & Troubleshooting:**
Both release generation and backup creation/restoration synchronously block the MCP/CLI handler and write a single canonical `AuditRow` to the system ledger when finished.
- *Limitations*: The tasks do not stream active progress via `stdout`. For very large directories (like `/var/aios`), the MCP call may hang for several minutes.
- *Error Diagnostics*: If the underlying OS tool (`genisoimage` or `zip`) fails, its native OS `stderr` buffer is losslessly captured (up to a safe 4KB limit to prevent ledger inflation) and injected straight into the `outcome_detail` of the ledger row. Operators can trace exact OS-level packaging failures strictly via the `aiosh ledger tail` command.
```bash
# Example: Run all Release Packaging & Backup tests (natively)
cargo test -p aiosh-core release
```

**Known Limitations (Release & Backup):**
1. **Windows Compatibility:** ISO generation mocks the invocation of `genisoimage` on Windows architectures due to missing GNU dependencies, outputting a dummy signature for hash continuity.
2. **Path Constraints:** Backup zipper strictly drops symbolic links and files exceeding 2GB (preventing infinite loops and disk saturation) during snapshot walks. Zip restoration enforces a strict Zip-Slip guard, ignoring any relative (`../`) or absolute paths.
3. **Rust Parity:** The core substrate is primarily maintained in Python (`aiosh-mcp`) while Windows Rust dependencies (`zip`, `libc`) receive stabilization updates. `cargo run` inside `aiosh-cli` intentionally fails on Windows due to `sandbox.rs` constraints (as designed).
4. **Configuration Limits:** The JSON configuration file is hard-capped at 64KB to prevent OOM DoS attacks. Malicious paths (e.g., `..` or absolute paths) in `output_dir` are strictly rejected by the config loader.
5. **Recovery Limits:** Backup archives exceeding 100,000 files or 10 GB total uncompressed size will be strictly rejected during restore to prevent zip bomb resource exhaustion. Restore targets must be empty.

Evidence: `tasks/evidence/T-00231-cli-surface-research.md` .. `T-00239-cli-surface-documentation.md` (and prior physical logic evidence `T-00211` .. `T-00230`), plus configuration evidence `tasks/evidence/T-00251-configuration-research.md` .. `tasks/evidence/T-00260-configuration-verification.md` and recovery & validation evidence `tasks/evidence/T-00301-recovery-validation-research.md` .. `tasks/evidence/T-00310-recovery-validation-verification.md`.

## Dependency & Toolchain Pinning (T-00311..T-00410)

AIOS relies on strict toolchain pinning to guarantee deterministic, reproducible, and verifiable builds across the ethical hacking platform. The `ToolchainManifest` acts as the governance overlay for native ecosystems (e.g. `Cargo.lock`, `rust-toolchain.toml`, `pyproject.toml`, `.nvmrc`).

**Usage Example (CLI):**
```bash
# Display the active toolchain configuration:
aiosh toolchain show

# Enforce active toolchain against the manifest configuration:
aiosh toolchain check

# Enforce using a custom toolchain configuration file:
aiosh toolchain check --config /path/to/custom_toolchain.json
```

**Usage Example (MCP):**
```json
// Example: Querying the active toolchain manifest via MCP
{
  "name": "aios.toolchain.config.get",
  "arguments": {}
}

// Example: Enforcing host environment against toolchain manifest via MCP
{
  "name": "aios.toolchain.check",
  "arguments": {}
}
```

**Configuration:**
Config is loaded in Rust from `$AIOSH_TOOLCHAIN_CONFIG` or `config/toolchain.json`. The file strictly declares versions for Rust, Python, and Node, and whether hash verification is enforced.

```bash
# Example: Provide custom toolchain configuration
echo '{"rust_version": "1.99.0", "python_version": "3.14", "node_version": "v24.18", "enforce_hashes": false}' > /tmp/toolchain.json
export AIOSH_TOOLCHAIN_CONFIG=/tmp/toolchain.json
```

**Automated Tests:**
The epic provides standalone smoke test suites for both CLI and MCP surfaces, wired into the centralized CI runner (`tools/ci_suites.py`):
```bash
# Run CLI toolchain smoke suite:
python3 code/aiosh-cli/tests/test_toolchain_cli_smoke.py

# Run MCP toolchain smoke suite:
python3 code/aiosh-mcp/tests/test_toolchain_mcp_smoke.py
```

**Security Policy (PEP Gating & Audit):**
Toolchain verification operations (`aiosh toolchain check`, `aios.toolchain.check`, `aiosh toolchain show`, `aios.toolchain.config.get`) are read-only and write immutable rows to the system audit ring. Any state-mutating toolchain commands (`aios.toolchain.set`, `toolchain.set`) are classified as `is_irreversible` and strictly gated behind cryptographic PEP grants (`requires_grant = true`). Unauthorized mutation attempts are refused synchronously and logged to the audit WAL.

**Recovery & Validation:**
- **Structural Validation (`validate_toolchain_manifest`)**: Evaluates manifest syntax and field integrity without spawning compiler subprocesses.
- **Default Recovery (`recover_default_toolchain`)**: Yields an in-memory canonical compile-time default manifest when disk configuration is corrupted or unreadable.
- **Drift Reconciliation (`reconcile_toolchain`)**: Probes host compilers and produces structured remediation recommendations (e.g. `rustup default 1.99.0`) for any drifted tools.

**Observability & Telemetry Diagnostics:**
Toolchain queries and enforcement runs capture rich runtime telemetry (`ToolchainTelemetry`):
- Provenance source tracking (`source: "default" | "file" | "env"`) is emitted in JSON envelopes by `aiosh toolchain show` and `aios.toolchain.config.get`.
- Version mismatch diagnostics are recorded losslessly in `outcome_detail` within SQLite WAL audit entries, viewable via `aiosh audit tail`.
- Subprocess probe stdout outputs are automatically clamped to 512 bytes with `[TRUNCATED]` markers to prevent log inflation.

**Known Limitations (Toolchain Pinning):**
1. **Subprocess Version Probing:** Toolchain checking probes host binaries (`rustc -V`, `python3 -V`, `node -v`) with a 15-second wall-clock timeout and process reap fallback to handle cold starts and rustup wrappers.
2. **Configuration Limits:** Like the release config, the JSON configuration file is hard-capped at 64KB to prevent OOM DoS attacks.
3. **Optional Node Verification:** The Node version (`node_version`) is marked as `Option<String>` to support legacy or minimal environments where the UI desktop layer is not active.

Evidence: `tasks/evidence/T-00311-data-model-research.md` .. `tasks/evidence/T-00408-recovery-validation-hardening.md`.


## Documentation Index Control (T-00411..T-00500)

Documentation Index Control provides native data structures and validation tooling in `aiosh-core` for tracking, indexing, and validating the document graph across the AIOS repository.

**Data Model (`DocIndexEntry` & `DocIndexManifest`):**
- **`DocIndexEntry`**: Represents an individual document node with `path`, `title`, `section`, optional `task_range`, and outbound `links`.
- **`DocIndexManifest`**: Aggregates entries with catalog `version`, providing query helpers (`find_entry_by_path`, `find_entries_by_section`) and integrity validation (`validate()`).

**JSON Representation Example:**
```json
{
  "version": "1.0.0",
  "entries": [
    {
      "path": "docs/README.md",
      "title": "Main Documentation",
      "section": "Overview",
      "task_range": "T-00001..T-00500",
      "links": ["docs/tasks/GOALS.md"]
    },
    {
      "path": "docs/tasks/GOALS.md",
      "title": "Goals & Laws",
      "section": "Governance",
      "task_range": null,
      "links": []
    }
  ]
}
```

**Hardening & Boundaries:**
- Enforces non-empty string invariants for path, title, and section fields.
- Rejects duplicate path entries to prevent documentation shadow collisions.
- Caps maximum manifest entries at 10,000 and maximum outbound links per entry at 1,000.
- Implements a 16 MiB per-file read limit (`MAX_DOC_BYTES`) to prevent denial-of-service memory spikes.

**Core Service Operations (`doc_index_service`):**
- **Document Indexing (`build_doc_index_from_paths`)**: Parses title (`# Title`) and relative markdown links from disk bounded by 16 MiB read caps.
- **Link Graph Validation (`validate_doc_links`)**: Scans manifest links against the repository root, detecting missing destination files and out-of-bounds traversal escapes.

**CLI Surface (`aiosh doc`):**
- `aiosh doc show [--json]`: Displays catalog of indexed documentation entries.
- `aiosh doc check [--repo <path>] [--json]`: Validates internal link integrity across documentation files.
- `aiosh doc search <query> [--json]`: Searches indexed documents by path, title, or section.

**MCP/API Surface (`aiosh-mcp`):**
- `aios.doc.index.get`: Returns the parsed documentation index manifest catalog.
- `aios.doc.check`: Validates documentation graph links, returning broken link reports.
- `aios.doc.search`: Queries indexed documentation entries with keyword filter.

**Configuration (`DocIndexConfig`):**
- Schema defined in `code/aiosh-rust/aiosh-core/src/doc_index_config.rs`.
- Configurable via file (`docs/doc_index_config.json`), `--config <path>`, or `AIOS_DOC_INDEX_CONFIG`.
- Supports `root_dirs`, `include_extensions`, `exclude_patterns`, and `enforce_strict_links`.

**Automated Tests:**
- Test runner: `python3 tools/test_doc_index_suites.py` (criteria D1..D7).
- Standalone smoke suites:
  - `python3 code/aiosh-cli/tests/test_doc_cli_smoke.py`
  - `python3 code/aiosh-mcp/tests/test_doc_mcp_smoke.py`
- Unit tests: `python3 tools/test_doc_index_unit.py` (U01..U13 + sensitivity proof S01).

**Security Policy & PEP:**
- Evaluated via `check_doc_index_policy()`.
- Read-only actions execute unauthenticated; mutating actions (`aios.doc.set`, `doc.set`) require active verified PEP grant tokens.
- Bounded file read (16 MiB max) and config read (64 KiB max) prevent denial-of-service memory exhaustion.
- Enforces repository checkout containment; out-of-bounds relative paths (`..`) are rejected and audited.
- CI policy verification enforced via `python3 tools/check_security_policy.py`.

**Observability & Diagnostics:**
- Data model: `DocIndexTelemetry` (`total_docs_indexed`, `total_links_checked`, `broken_links_count`, `is_healthy`).
- Emitted in CLI (`aiosh doc check --json`) and MCP JSON-RPC (`aios.doc.check`) responses.
- Structured telemetry and validation reports are persisted to the SQLite WAL audit ring on every invocation.
- Example CLI diagnostic invocation:
  ```bash
  aiosh doc check --json
  ```

**Recovery & Validation:**
- Default configuration recovery: `recover_default_doc_index_config()` restores strict in-memory defaults when config files are absent or malformed.
- Catalog validation: `validate_doc_index_catalog()` runs pure link graph verification returning structured telemetry or detailed failure reasons.
- Atomic reconciliation: `reconcile_doc_index()` reads, indexes, validates, and generates telemetry in a single idempotent pass.

Evidence: `tasks/evidence/T-00411-data-model-research.md` .. `tasks/evidence/T-00508-recovery-validation-hardening.md`.


## Evidence & Audit Trail (T-00511..T-00610)

Evidence & Audit Trail establishes data models and verification infrastructure in `aiosh-core` for tracking, validating, and cryptographically auditing task execution artifacts and their associated SHA-256 evidence records.

**Data Model (`EvidenceRecord` & `TaskEvidenceManifest`):**
- **`EvidenceStep`**: Defines the 10 sub-epic lifecycle phases (`Research`, `Spec`, `Scaffold`, `Implementation`, `UnitTest`, `Integration`, `SecurityReview`, `Hardening`, `Documentation`, `Verification`).
- **`EvidenceRecord`**: Tracks an individual task artifact with `task_id`, `step`, `file_path`, `sha256_hash`, `timestamp_utc`, `status` (`pass` | `fail` | `pending`), and optional `summary`.
- **`TaskEvidenceManifest`**: Groups records under an `epic_name` and `task_range`, providing serialization to canonical JSON and querying methods (`get_record`, `filter_by_step`).
- **`EvidenceVerificationReport`**: Captures totals, missing files, checksum mismatches, and overall boolean validation status.

**Invariants & Bounds:**
- Task ID bounded between 1 and 10000.
- Strict relative path checks; absolute paths and `..` traversal escapes are rejected.
- Checksums strictly validated as 64-character lowercase hexadecimal SHA-256 strings.
- Enforces uniqueness of `(task_id, step)` pairs within manifests.
- Manifest record capacity capped at 10,000 entries.

**Core Service Operations (`evidence_service`):**
- **SHA-256 Checksum Computation (`compute_file_sha256`)**: Computes deterministic lowercase hex SHA-256 strings for files on disk bounded by 16 MiB read caps.
- **Evidence Record Construction (`build_evidence_record`)**: Reads and hashes evidence artifacts, generating validated `EvidenceRecord` items.
- **Manifest Verification (`verify_evidence_manifest`)**: Validates every artifact in a manifest against on-disk files, compiling missing files and checksum mismatches into an `EvidenceVerificationReport`.

**CLI Surface (`aiosh evidence`):**
- `aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]`: Verifies evidence files against the manifest.
- `aiosh evidence hash <path> [--json]`: Computes and displays the SHA-256 checksum of the target file.
- `aiosh evidence scan [--repo <path>] [--task <id>] [--json]`: Discovers and indexes evidence files in `docs/tasks/evidence/`.
- Example invocations:
  ```bash
  aiosh evidence hash docs/README.md --json
  aiosh evidence verify --json
  aiosh evidence scan --task 501 --json
  ```

**MCP/API Surface (`aiosh-mcp`):**
- `aios.evidence.verify`: Runs evidence manifest verification and returns `report`.
- `aios.evidence.hash`: Computes SHA-256 checksum for a file on disk.
- `aios.evidence.scan`: Discovers and indexes evidence files in `docs/tasks/evidence/` with optional task filtering.
- Example MCP `tools/call` JSON-RPC payload:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "aios.evidence.scan",
      "arguments": { "repo_path": ".", "task_id": 501 }
    }
  }
  ```

**Configuration Model (`EvidenceConfig`):**
- Standard repository configuration file: `config/evidence.config.json`.
- Configurable settings: `evidence_dir` (default: `"docs/tasks/evidence"`), `max_file_bytes` (default: 16 MiB, max 64 MiB), `allowed_extensions` (`[".md", ".json"]`), `enforce_checksum` (boolean).
- Precedence: `AIOS_EVIDENCE_CONFIG_PATH` > `AIOS_EVIDENCE_DIR` / `AIOS_EVIDENCE_MAX_FILE_BYTES` > `config/evidence.config.json` > in-memory defaults.
- Guardrails: Maximum 64 KiB configuration file size to prevent config poisoning.

**Security Policy & PEP:**
- Evaluated via `check_evidence_policy()`.
- Read-only actions (`aios.evidence.verify`, `evidence.verify`, `aios.evidence.hash`, `evidence.hash`, `aios.evidence.scan`, `evidence.scan`) execute unauthenticated.
- Mutating actions (`aios.evidence.record`, `evidence.record`, `aios.evidence.set`, `evidence.set`) require active verified PEP grant tokens. Missing or empty tokens fail closed with `PermissionDenied`.
- Refusals produce structured error responses and append an honest `outcome="refused"` audit row to SQLite WAL.
- Enforces repository checkout containment; out-of-bounds paths are rejected and audited.

**Automated Tests & CI Verification (`check_evidence.py` / `test_check_evidence.py`):**
- Invariant criteria:
  - `E1 (directory-health)`: `docs/tasks/evidence/` exists and contains valid markdown artifacts.
  - `E2 (ledger-consistency)`: Completed tasks recorded in `TASK_STATE.json` have corresponding files on disk.
  - `E3 (file-bounds)`: All evidence files are non-empty, valid UTF-8, and strictly $\le 16\text{ MiB}$.
  - `E4 (hash-consistency)`: Deterministic lowercase SHA-256 digests.
- Standalone execution:
  ```bash
  python tools/check_evidence.py
  python tools/test_check_evidence.py
  ```
- Registered CI suites: `evidence_cli_smoke`, `evidence_mcp_smoke`, `evidence_checker`, `evidence_unit`.
- Constraints & Limitations: File reads are capped at 16 MiB; large ledger validation utilizes bounded task sampling to maintain rapid CI turnaround.

**Observability & Diagnostics (`EvidenceTelemetry`):**
- Schema: `total_records`, `valid_records`, `missing_files_count`, `hash_mismatches_count`, `is_healthy`.
- Helper: `collect_evidence_telemetry(&report)`.
- Verification CLI telemetry invocation:
  ```bash
  aiosh evidence verify --json
  ```
- Verification output example:
  ```json
  {
    "status": "ok",
    "is_valid": true,
    "total_records": 10,
    "valid_records": 10,
    "missing_files_count": 0,
    "hash_mismatches_count": 0,
    "is_healthy": true
  }
  ```
- Audit logging applies 512-byte outcome string clamping (`clamp_str`) for database protection.
- Human-readable manifest summary formatting is available via `format_evidence_summary(&manifest)`.
- Manifest recovery and live disk reconciliation are provided via `recover_default_evidence_config()`, `reconstruct_evidence_manifest(&repo, range, epic)`, and `reconcile_evidence_manifest(&repo, &manifest)`.

Evidence: `tasks/evidence/T-00511-data-model-research.md` .. `tasks/evidence/T-00609-recovery-validation-documentation.md`.


## Repository Health (T-00611..T-00710)

The **Repository Health** subsystem provides automated health, hygiene, and governance diagnostics across all repository substrates (Git tree cleanliness, file bounds, line ending hygiene, security governance, and workspace limits).

**Data Model (`aiosh-core::repo_health`):**
- `HealthStatus`: Discrete assessment level (`Pass`, `Warn`, `Fail`, `Skip`).
- `HealthCategory`: Check domains (`GitHygiene`, `FileIntegrity`, `SecurityGovernance`, `DependencyHygiene`, `WorkspaceBounds`).
- `RepoHealthCheck`: Granular assessment record with `check_id` (`[a-zA-Z0-9_-]+`), `name`, `category`, `status`, `message`, `details`, and `duration_ms`.
- `RepoHealthReport`: Aggregated report capturing `repo_path`, `timestamp_utc`, `overall_status`, `total_checks`, `passed_checks`, `warn_checks`, `failed_checks`, `skipped_checks`, and list of `checks`.

**Core Service Operations (`aiosh-core::repo_health_service`):**
- **Git Working Tree Inspection (`check_git_working_tree`)**: Executes `git status --porcelain=v2`, parsing tracked and untracked modifications into structured `RepoHealthCheck` records.
- **File Bounds Scan (`check_file_bounds`)**: Recursively verifies repository files within a configurable limit (default 16 MiB), ignoring build directories (`.git/`, `target/`, `node_modules/`, `.venv/`).
- **Security Governance Audit (`check_security_governance`)**: Validates root `SECURITY.md` existence, length, and absence of unresolved placeholder markers.
- **Health Orchestrator (`check_repo_health`)**: Executes all diagnostic suites and synthesizes a validated `RepoHealthReport`.
- **Human-Readable Formatter (`format_repo_health_summary`)**: Renders console and log summaries of `RepoHealthReport` with status banners, per-check elapsed timings, detail clamping (50 items max), and aggregate statistics.
- **Recovery & Validation (`reconcile_repo_health`, `recover_default_repo_health_config`, `reconstruct_repo_health_report`)**: Zero-downtime diagnostic recovery, resilient fallback to canonical in-memory configuration, and mathematical invariant report validation.

**Configuration (`aiosh-core::repo_health_config`):**
- `RepoHealthConfig`: Configurable parameters governing health evaluations (`max_file_bytes`, `ignored_dirs`, `require_clean_git`, `security_policy_path`, `min_security_policy_bytes`).
- Resolved via explicit file path, `AIOS_REPO_HEALTH_CONFIG` environment variable, or `docs/repo_health_config.json`.

```json
{
  "version": "1.0.0",
  "max_file_bytes": 16777216,
  "ignored_dirs": [".git", "target", "node_modules", ".venv"],
  "require_clean_git": false,
  "security_policy_path": "SECURITY.md",
  "min_security_policy_bytes": 100
}
```
**Automated Test Suite (`tools/test_repo_health_suites.py`):**

Standalone test runner covering criteria H1..H7:

```bash
python tools/test_repo_health_suites.py
# [+] H1 data model integrity
# [+] H2 git tree hygiene diagnostics
# [+] H3 file bounds scanner
# [+] H4 security governance audit
# [+] H5 CLI surface commands
# [+] H6 MCP tool schemas & JSON-RPC
# [+] H7 configuration schema & hardening
# PASS: repo_health_suites criteria (H1..H7)
```

**Observability & Diagnostics:**
- `RepoHealthReport` includes `duration_ms` per check and aggregate counters (`total_checks`, `passed_checks`, `warn_checks`, `failed_checks`, `skipped_checks`).
- Structured diagnostic output is accessible via CLI (`aiosh repo health [--json]`) and MCP tool invocation (`aios.repo.health`).

Evidence: `tasks/evidence/T-00611-data-model-research.md` .. `tasks/evidence/T-00709-recovery-validation-documentation.md`.


## Secrets & Access Hygiene (T-00711..T-00810)

Secrets & Access Hygiene provides high-precision detection, categorization, and redaction of credentials, private keys, and API tokens across the AIOS codebase and workspace.

**Data Model (`aiosh-core::secrets`):**
- `SecretSeverity`: Discrete classification (`Critical`, `High`, `Medium`, `Low`, `Info`).
- `SecretPatternKind`: Pattern family (`PrivateKey`, `ApiToken`, `AwsCredentials`, `PasswordInConfig`, `HighEntropyGeneric`).
- `SecretFinding`: Granular finding record containing `rule_id`, `path`, `line_number`, `severity`, `pattern_kind`, `description`, `redacted_snippet`, and `fingerprint`.
- `SecretScanReport`: Aggregated report tracking `repo_path`, `timestamp_utc`, `is_clean`, findings counts, and findings list.
- `redact_secret_value`: Safe redaction utility preserving boundary characters for strings $\ge 12$ chars with `****` masking.

**Core Service Operations (`aiosh-core::secrets_service`):**
- **File Scanner (`scan_file_for_secrets`)**: Scans a single target file for private key blocks (`SEC-001`), AWS credentials (`SEC-002`), GitHub tokens (`SEC-003`), generic API keys (`SEC-004`), and password assignments (`SEC-005`), skipping binary content.
- **Workspace Scanner (`scan_workspace_for_secrets`)**: Recursively crawls directory trees, filtering build artifacts (`.git`, `target`, `node_modules`, `.venv`, `dist`) and aggregating findings into a validated `SecretScanReport`.

**CLI Subcommand Surface (`aiosh-cli`):**
- `aiosh secrets scan [--repo <path>] [--file <path>] [--json] [--max-bytes <n>]`: Detailed scan outputting finding cards with redacted snippets and sha256 fingerprints.
- `aiosh secrets check [--repo <path>] [--json]`: Fast boolean pass/fail verification for CI gates.

```bash
aiosh secrets scan --file code/aiosh-rust/Cargo.toml
# === Secrets & Access Hygiene Scan: code/aiosh-rust/Cargo.toml ===
# Status: CLEAN (1 files scanned, 0 findings: 0 critical, 0 high, 0 medium, 0 low)

aiosh secrets check --json
# { "ok": true, "subcommand": "secrets check", "data": { ... } }
```

**MCP Tool Integration (`aiosh-mcp`):**
- `aios.secrets.scan`: JSON-RPC 2.0 tool scanning workspace or single file for exposed secrets without exposing raw credentials.
- `aios.secrets.check`: Fast boolean verification returning `{ "ok": true, "is_clean": bool, "total_findings": u32 }`.

```json
// tools/call -> aios.secrets.scan
{
  "name": "aios.secrets.scan",
  "arguments": { "repo_path": ".", "max_bytes": 16777216 }
}
```

**Configuration (`aiosh-core::secrets_config` / `docs/secrets_config.json`):**
- Schema-validated configuration `SecretsConfig` with versioning, custom file/line bounds, ignored directories, and allowlist patterns.
- Loading precedence: `--config <path>` $\to$ `AIOS_SECRETS_CONFIG` $\to$ `docs/secrets_config.json` $\to$ `SecretsConfig::default()`.

```json
{
  "version": "1.0.0",
  "max_file_bytes": 16777216,
  "max_line_bytes": 4096,
  "ignored_dirs": [".git", "target", "node_modules", ".venv", "dist"],
  "allow_patterns": [],
  "require_clean": false
}
```

**Automated Test Suite (`tools/test_secrets_suites.py`):**

Standalone test runner covering criteria K1..K9:

```bash
python tools/test_secrets_suites.py
# [+] K1 data model integrity
# [+] K2 private key scanner
# [+] K3 API token scanner
# [+] K4 config & env credentials scanner
# [+] K5 CLI surface commands & options
# [+] K6 MCP tool schemas & execution
# [+] K7 SecretsConfig schema, validation & roundtrip
# [+] K8 observability & scan telemetry
# [+] K9 recovery & report validation invariants
# PASS: secrets_suites criteria (K1..K9)
```

**Observability & Telemetry (`aiosh-core::secrets`):**
- `SecretScanReport::severity_counts()` returns quantitative breakdown `(critical, high, medium, low)`.
- `SecretScanReport::summary_line()` produces human-readable diagnostic line for CLI and log streams.

**Recovery & Validation Protocol (`aiosh-core::secrets`):**
- `validate_secret_report`: Enforces invariant integrity across total findings, severity sums, and cleanliness flags.
- Contaminated repository recovery: Revoke compromised keys at provider, remove plaintext tokens from source files, and configure allowlists in `docs/secrets_config.json` for verified synthetic fixtures.

**Security Policy & Threat Mitigation (`SECURITY.md`):**
- Prohibits committing raw credentials or private keys; mandates redaction via `redact_secret_value` and cryptographic SHA-256 fingerprinting.
- Validated via `tools/check_security_policy.py` (criteria S1..S5).

Evidence: `tasks/evidence/T-00711-data-model-research.md` .. `tasks/evidence/T-00809-recovery-validation-documentation.md`.


## Regression Triage (T-00811..T-00910)

Automated test failure categorization, fingerprinting, deduplication, and regression lifecycle triage for the AIOS userspace ecosystem.

**Core Data Model (`aiosh-core::triage`):**
- `TriageStatus`: Lifecycle states (`Untriaged`, `Triaged`, `FixPending`, `Resolved`, `WontFix`).
- `TriageSeverity`: Impact classification (`Blocker` / P0, `Critical` / P1, `Major` / P2, `Minor` / P3).
- `TriageRecord`: Granular failure struct containing deduplication SHA-256 `signature`, `test_target`, `error_message`, `repro_command`, `occurrences`, timestamps, and optional blame (`blame_task_id`, `blame_commit`).
- `TriageReport`: Report aggregating active vs resolved regressions with `validate_triage_report` validation.

**Core Service (`aiosh-core::triage_service`):**
- `TriageStore`: In-memory and file-persisted store with signature and ID indexing.
- `ingest_ci_summary`: Automated ingestion and correlation from `ci::RunSummary`.
- Deduplication and lifecycle transitions (`Untriaged` -> `Triaged` -> `FixPending` -> `Resolved`, reopening on recurrence).
- 1 MiB hard size cap on store JSON files.

**CLI Surface (`aiosh triage`):**

```bash
aiosh triage list [--status <st>] [--severity <sev>] [--json] [--store <path>]
aiosh triage show <id> [--json] [--store <path>]
aiosh triage record --target <target> --suite <suite> --error <msg> [--repro <cmd>] [--severity <sev>]
aiosh triage resolve <id> --notes <notes>
aiosh triage ingest <summary_json_file>
aiosh triage check [--store <path>]
```

**MCP / JSON-RPC API Surface:**
- `aios.triage.list`: List triage records with optional `status`, `severity`, and `store_path`.
- `aios.triage.show`: Show detailed record metadata for given `id`.
- `aios.triage.record`: Record a regression finding (`test_target`, `suite_name`, `error_message`, `repro_command`, `severity`).
- `aios.triage.resolve`: Mark a regression as resolved (`id`, `notes`).
- `aios.triage.check`: Cleanliness check verifying that no open blocker/critical regressions exist.

**Configuration (`aiosh-core::triage_config`, `docs/triage_config.json`):**
- `TriageConfig`: Enforces bounded parameters (`max_store_bytes` between 16 KiB and 64 MiB, `retention_days` $\ge 1$, `auto_ingest_suites` patterns, `default_severity`, `notify_blockers`).
- Loaded via `--config <path>`, `$AIOS_TRIAGE_CONFIG`, or deterministic default.
- Bounded file size cap (`MAX_CONFIG_FILE_BYTES = 65536`).

**Automated Test Runner (`tools/test_triage_suites.py` & `tools/test_triage_unit.py`):**

```bash
python tools/test_triage_suites.py
# [+] T1 triage data model integrity & failure signatures
# [+] T2 triage store, CI summary ingestion & persistence
# [+] T3 CLI surface commands, flags & flow
# [+] T4 MCP surface tools, params & flow
# [+] T5 triage configuration schema, validation & filters
# [+] T6 end-to-end regression triage lifecycle & recurrence
# [+] T7 triage observability summary metrics & lifecycle diagnostics
# [+] T8 triage recovery resilience, error handling & invariant validation
# PASS: triage_suites criteria (T1..T8)
```

**Observability & Telemetry (`aiosh-core::triage`):**
- `TriageReport::status_counts`: Breakdown across `Untriaged`, `Triaged`, `FixPending`, `Resolved`, and `WontFix`.
- `TriageReport::severity_counts`: Breakdown across `Blocker`, `Critical`, `Major`, and `Minor`.
- `TriageReport::summary_line`: Standardized single-line summary string for CLI and logs.

**Recovery & Validation Engine (`aiosh-core::triage`, `aiosh-core::triage_service`):**
- `validate_triage_record`: Structural invariant checking on ID format, SHA-256 signature, non-empty fields, and occurrence count.
- `TriageStore::load_or_recover`: Resilient store loading gracefully recovering from corrupted/invalid JSON files with diagnostic warnings.

**Security Policy & Threat Mitigation (`SECURITY.md`):**
- Prohibits forging, tampering with, or bypassing regression triage records to mask blocker or critical regressions.
- All state-changing triage commands emit immutable audit records to SQLite WAL.
- Validated via `tools/check_security_policy.py` (criteria S1..S5) and `docs/tasks/evidence/T-00877-security.md`.

Evidence: `tasks/evidence/T-00811-data-model-research.md` .. `tasks/evidence/T-00909-recovery-validation-documentation.md`.


## Agent Handoff Protocol (T-00911..T-01000)

Agent Handoff Protocol provides a tamper-evident, auditable mechanism for transferring
execution context, credentials, and state between autonomous agents or between human
supervisors and subagents.

**Data Model (`aiosh-core::handoff`):**
- `HandoffRecord`: Stores `id` (`HND-<hash>`), `sender_agent_id`, `receiver_agent_id`, optional `task_id`, `context_summary`, `payload_json`, `priority` (`low`, `normal`, `high`, `urgent`), `status` (`pending`, `accepted`, `rejected`, `completed`, `cancelled`, `expired`), `created_at`, `expires_at`, and `signature`.
- `HandoffReport`: Aggregated summary with `total_handoffs`, `active_handoffs`, and `completed_handoffs`.
- `compute_handoff_signature`: Deterministic SHA-256 fingerprint over normalized sender, receiver, task ID, and payload.

**Core Service Store (`aiosh-core::handoff_service`):**
- `HandoffStore`: In-memory state store managing active handoff queues, deduplication, and atomic persistence.
- State transitions: `initiate_handoff`, `accept_handoff`, `reject_handoff`, `complete_handoff`, and `cancel_handoff`.
- Persistence: Atomic temporary file write (`.tmp`) with atomic rename and corruption recovery fallback (`load_or_recover`).

**CLI Surface (`aiosh handoff`):**
- `aiosh handoff list [--active] [--status <status>] [--json] [--store <path>]`: Lists tracked handoffs.
- `aiosh handoff show <id> [--json] [--store <path>]`: Displays full details and context for a specific handoff.
- `aiosh handoff initiate --sender <S> --receiver <R> [--task <T>] --summary <CTX> [--payload <JSON>] [--priority <P>]`: Initiates handoff.
- `aiosh handoff accept <id> [--notes <notes>]`: Accepts a pending handoff.
- `aiosh handoff reject <id> [--notes <notes>]`: Rejects a pending handoff.
- `aiosh handoff complete <id> [--notes <notes>]`: Completes an active handoff.
- `aiosh handoff cancel <id> [--notes <notes>]`: Cancels a pending/active handoff.

**MCP/API Surface (`aiosh-mcp`):**
- `aios.handoff.list`: Queries active or historical handoffs with optional status/active filtering.
- `aios.handoff.show`: Fetches full metadata and payload for a specific handoff ID.
- `aios.handoff.initiate`: Enqueues a new handoff between sender and receiver agents.
- `aios.handoff.accept`, `aios.handoff.reject`, `aios.handoff.complete`, `aios.handoff.cancel`: Manages handoff lifecycle state transitions.
- Example MCP `tools/call` JSON-RPC payload:
  ```json
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "aios.handoff.initiate",
      "arguments": {
        "sender": "planner",
        "receiver": "executor",
        "summary": "Execute implementation phase",
        "priority": "high"
      }
    }
  }
  ```

**Configuration (`HandoffConfig` & `docs/handoff_config.json`):**
- `max_store_bytes`: Maximum allowed size for `handoff_store.json` (bounds: 16 KiB to 64 MiB, default: 1 MiB).
- `default_priority`: Default handoff priority level (`Normal`).
- `default_ttl_seconds`: Expiration timeout for unaccepted handoffs (default: 86,400s / 24h).
- `allow_auto_accept`: Flag indicating whether receiving agents may auto-accept tasks (default: `false`).
- Config loading order: Explicit CLI flag `--config` $\to$ `AIOSH_HANDOFF_CONFIG` env $\to$ `docs/handoff_config.json` $\to$ in-memory defaults.

**Security Policy (`verify_handoff_authorization` & PEP Integration):**
- Role-based authorization: Receiver agents are authorized to `accept`, `reject`, and `complete`; sender agents are authorized to `cancel`.
- Global operator override: `operator`, `admin`, and `root` actors have universal intervention authority.
- Unauthorized interception: Returns explicit `PermissionDenied` error without state modification.

**Observability (`HandoffReport` & Metrics Aggregation):**
- `HandoffReport`: Aggregated snapshot providing `total_handoffs`, `active_handoffs`, and `completed_handoffs`.
- Arithmetic invariant: `active_handoffs + completed_handoffs == total_handoffs` enforced by `validate_handoff_report`.
- Timestamping: ISO-8601 UTC timestamp tracking report generation time.

**Standalone Test Runner (`tools/test_handoff_suites.py`):**
```bash
python tools/test_handoff_suites.py
# [+] H1 handoff data model integrity & signature determinism
# [+] H2 handoff store lifecycle, transitions & persistence
# [+] H3 handoff CLI surface subcommands & flow
# [+] H4 handoff MCP surface tools & flow
# [+] H5 handoff configuration schema, validation & roundtrip
# [+] H6 handoff automated edge cases, state matrix & batch fuzzing
# [+] H7 handoff security policy & actor authorization matrix
# [+] H8 handoff observability metrics, status aggregation & reports
# PASS: handoff_suites criteria (H1..H8)
```

Evidence: `tasks/evidence/T-00911-data-model-research.md` .. `tasks/evidence/T-00999-documentation-documentation.md`.


### 8.10 Distro Selection & Justification (Phase 1, T-01001..T-01100)

AIOS evaluates, specifies, and builds upon a lightweight, reproducible Linux base operating environment.
Detailed architectural specification and justification guide: `docs/distro_selection.md`.

**Data Model (`DistroProfile`, `DistroEvaluation`, `DistroFamily`, `InitSystem`, `ArchTarget`, `CLibrary`):**
- Primary Base (`Debian 12 Bookworm Minimal`): `glibc`, `systemd` (cgroup v2), Python AI/ML binary wheel compatibility, >= 6.1 LTS kernel baseline.
- Container Base (`Alpine Linux 3.19`): `musl`, ultra-compact footprint (<10MB) for isolated ephemeral worker sandboxes.
- Production readiness formula: `0.4 * binary_compat + 0.3 * security + 0.3 * footprint >= 0.75`.

**Core Service (`DistroStore` in `aiosh-core::distro_service`):**
- In-memory registry with atomic disk persistence (`save_to_path`, `load_from_path`, `load_or_recover`).
- Hard size cap of 10 MiB (`MAX_STORE_BYTES`) to prevent memory exhaustion attacks.
- Atomic file write via `.tmp.<pid>` with defensive error cleanup.

**CLI Surface (`aiosh distro`):**
```bash
aiosh distro list [--json] [--store <path>]
aiosh distro show <id> [--json] [--store <path>]
aiosh distro evaluate [<id>] [--json] [--store <path>]
aiosh distro recommend [--json] [--store <path>]
aiosh distro config [--json]
aiosh distro policy [<id>] [--json] [--store <path>]
aiosh distro stats [--json] [--store <path>]
```

**Configuration Subsystem (`config/distro.json` & `aiosh-core::distro_config`):**
- Canonical configuration file: `config/distro.json`.
- Environment overrides: `AIOSH_DISTRO_CONFIG`, `AIOSH_DISTRO_STORE_PATH`, `AIOSH_DEFAULT_DISTRO`.
- Provenance reporting: `aiosh distro config --json` reports origin (`env`, `file`, or `default`) for all properties.
- Security bounds: Capped at 64 KiB (`take(65_536)`), rejection of IEEE 754 `NaN`, and directory traversal (`..`) checks.

**Security Policy Subsystem (`aiosh-core::distro_policy`):**
- Contract: `DistroSecurityPolicy` enforcing minimum security scores ($\ge 0.70$), binary compatibility floors ($\ge 0.70$), and family exclusions.
- CLI evaluation: `aiosh distro policy [<id>] [--json]` reports compliance verdicts and diagnostics.
- MCP evaluation: `aios.distro.policy` audits compliance via PEP and audit ring buffer.
- Environment overrides: `AIOSH_DISTRO_MIN_SECURITY_SCORE`, `AIOSH_DISTRO_DISALLOWED_FAMILIES`.

**Observability Subsystem (`aiosh-core::distro_observability`):**
- Telemetry contract: `DistroObservabilityReport` tracking total profiles, recommended profile, production readiness, policy compliance, score averages, and family/arch partitions.
- Arithmetic invariants: O1..O4 validated upon construction (partition sums, count bounds, clamped score averages).
- CLI exposure: `aiosh distro stats [--json]` displays structured telemetry diagnostics.
- MCP exposure: `aios.distro.stats` delivers JSON telemetry payload via audit ring buffer.

**MCP / JSON-RPC API Surface:**
- `aios.distro.list`: List all registered distro profiles.
- `aios.distro.show`: Get detailed profile specification by ID.
- `aios.distro.evaluate`: Evaluate single profile or all profiles against AIOS criteria.
- `aios.distro.recommend`: Return the reference production profile.
- `aios.distro.policy`: Audit distro profiles against AIOS security policy standards.
- `aios.distro.stats`: Retrieve aggregated telemetry and observability report.

**Standalone Test Runner (`tools/test_distro_suites.py` & `tools/test_distro_unit.py`):**
```bash
python tools/test_distro_suites.py
# [+] D1 distro data model integrity & validation invariants
# [+] D2 distro store lifecycle, registry querying & persistence
# [+] D3 distro CLI surface commands & options (list/show/evaluate/recommend)
# [+] D4 distro MCP tools dispatch & execution (list/show/evaluate/recommend)
# [+] D5 distro configuration resolution & hardening invariants
# PASS: distro_suites criteria (D1..D5)
```

Limitations (honest): Profiles must pass strict semver validation and alphanumeric ID checks; custom profiles do not auto-build target ISO images (handled by downstream image building tasks); file loading is subject to a 10 MiB file cap; configuration file is subject to a 64 KiB cap.

```bash
python code/aiosh-cli/tests/test_distro_cli_smoke.py
# ALL DISTRO CLI SMOKE TESTS PASSED!

python code/aiosh-mcp/tests/test_distro_mcp_smoke.py
# ALL DISTRO MCP SMOKE TESTS PASSED!
```

Evidence: `tasks/evidence/T-01001-data-model-research.md` .. `tasks/evidence/T-01089-documentation-documentation.md`.




## Documentation invariants (Task Ledger Control, T-00091..T-00100)

`tools/check_task_docs.py` keeps THIS doc set rot-proof. Read-only,
stdlib-only, exit 0/1 — runs in CI as `task_docs_unit` +
`task_docs_scaffold` and standalone:

```bash
python3 tools/check_task_docs.py
# [✓] C1 spec-health        SPEC exists, marker-free
# [✓] C2 component sections ### 8.1..8.6 keep their frozen epic ranges
# [✓] C3 referenced paths   backticked docs/code/ci/tools paths resolve
#                           (fenced blocks + example x.md excluded)
# [✓] C4 phase map          MASTER_TASK_LEDGER.md table == JSONL phases
# [✓] C5 index health       marker-free docs; links stay inside checkout
# [✓] C6 no volatile counts living docs never embed "CI n/n" snapshots
# PASS: task docs criteria (C1..C6)
```

Limitations (honest): structural checks only — prose quality is human
judgment; C2's frozen section list grows monotonically when a new
component closes (add one entry + its range); C5's containment boundary
is the repo root (`../` links into the tree are fine, escapes are
flagged); reads are capped at 16 MiB; deliberately NOT exposed over
MCP (operator surface only).

Evidence chain: research `tasks/evidence/T-00091-research.md` · spec
T-00092 · scaffold T-00093 · implementation T-00094 · unit tests
T-00095 · integration T-00096 · security T-00097 · hardening
T-00098 · verification T-00100.

## Quick links

- [`../START_HERE.md`](../START_HERE.md)
- [`../PROGRESS_LOG`](../progress.md)
- [`../TASK_PLAN`](../task_plan.md)
- [`tasks/TASK_STATE.json`](tasks/TASK_STATE.json) — live task pointer
- [`tasks/MASTER_TASK_LEDGER.md`](tasks/MASTER_TASK_LEDGER.md) — task index
