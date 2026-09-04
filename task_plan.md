# AIOS Active Task Plan — v2 (course correction 2026-08-20)

## Goal (v2, amended)

Deliver the user-stated vision:

> **A Linux system for ethical hacking on the inside, a Windows-style desktop on
> the outside, with AI as a first-class S-rank kernel subsystem that controls
> the whole system.**

This supersedes the v1 plan that focused on a from-scratch RISC-V microkernel
as the shipping target. The microkernel work is preserved as research
substrate but is no longer the shipping path.

## Phase ordering — v2 critical path

1. **Phase 0 — Pillar C spine (S-rank AI subsystem).** Pluggable inference
   backends; MCP JSON-RPC server; PEP capability grants; audit ring.
   **This is the NEW critical path.**
2. **Phase 1 — Pillar A wrappers.** Top Kali/Parrot/BlackArch tools
   exposed as MCP tools, MITRE ATT&CK category-aligned.
3. **Phase 2 — Pillar B installer.** Debian/Ubuntu LTS + KDE Plasma 6 with
   Windows-look theme + Wine 11 + Proton 11.
4. **Phase 3 — AI ↔ Pillar A integration.** Goal-driven recon-to-report
   pipeline orchestrated by the S-rank agent.
5. **Phase 4 — AI ↔ Pillar B integration.** GUI automation over Wayland /
   KWin / UIA / AT-SPI.
6. **Phase 5 — Hardening, cross-platform, release.**

## Status (live — 2026-09-04)

### 2026-09-04 — MILESTONE: Base Image Build Observability CLOSED 10/10 (T-01171..T-01180)

Complete implementation, governance, verification, and hardening for Phase 1 `Base Image Build / observability`:
- **Observability Reporting Engine**: `BaseImageObservabilityReport` in `code/aiosh-rust/aiosh-core/src/base_image_observability.rs` aggregating total manifests, format distributions, arch distributions, distro breakdowns, policy compliance, unique kernel versions, and storage budgets.
- **Arithmetic Invariants (OB1..OB5)**: Complete validation enforcing format/arch/distro breakdown sum equality, policy compliance bounds, and average calculation integrity.
- **CLI & MCP Tool Surfaces**: `aiosh image report` and `aios.image.report` with structured JSON output envelopes and SQLite WAL audit logging.
- **Security Hardening**: Enforced bounds (16 formats, 64 archs, 256 distros, 256 kernels), control character rejection, and fail-closed error handling.
- **Test Runner Matrix (`tools/test_image_suites.py`)**: All criteria `B1..B8` PASS.
- **Ledger Pointer**: Advances to **T-01181** (`Phase 1 / Base Image Build / documentation: Research`).

### 2026-09-01 — MILESTONE: Distro Selection & Justification Data Model CLOSED 10/10 (T-01001..T-01010) — PHASE 1 INITIATED

Complete implementation, governance, verification, and hardening for Phase 1 `Distro Selection & Justification / data model`:
- **Distribution Model Specification**: `DistroProfile`, `DistroEvaluation`, `DistroFamily`, `InitSystem`, `ArchTarget`, `CLibrary` in `code/aiosh-rust/aiosh-core/src/distro.rs`.
- **Target Invariant Validation**: `validate_distro_profile` enforcing semver kernel parsing, profile ID character whitelisting, and field bounds.
- **Production Scoring Engine**: Multi-criteria weighted evaluation algorithm (`DistroEvaluation::evaluate`).
- **Test Runner Matrix (`tools/test_distro_suites.py`)**: Criterion `D1` distro data model integrity and validation.
- **Unit Suite (`tools/test_distro_unit.py`)**: Behavioral assertions U01..U04 passing in isolation.
- **Documentation**: Updated `docs/README.md` with Phase 1 section and distro profiles.
- **Ledger Pointer**: Advances to **T-01011** (`Phase 1 / Distro Selection & Justification / core service: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Documentation CLOSED 10/10 (T-00991..T-01000) — EPIC COMPLETE (100/100) — TASK 1000 ACHIEVED!

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / documentation`:
- **Full Epic Closure**: 100/100 tasks across Data Model (`T-00911..T-00920`), Core Service (`T-00921..T-00930`), CLI Surface (`T-00931..T-00940`), MCP/API Surface (`T-00941..T-00950`), Configuration (`T-00951..T-00960`), Automated Tests (`T-00961..T-00970`), Security Policy (`T-00971..T-00980`), Observability (`T-00981..T-00990`), and Documentation (`T-00991..T-01000`).
- **Comprehensive Documentation**: Complete coverage in `docs/README.md` with rot-proof invariants C1..C6.
- **Master Test Runner Matrix (`tools/test_handoff_suites.py`)**: All criteria `H1..H8` PASS.
- **Unit Suite (`tools/test_handoff_unit.py`)**: All assertions `U01..U17` PASS.
- **Milestone Metric**: **TASK 1000 / 10,000 (10.00%) REACHED!**
- **Ledger Pointer**: Advances to **T-01001** (`Phase 0 / Agent Coordination Protocol / data model: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Observability CLOSED 10/10 (T-00981..T-00990)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / observability`:
- **Observability Reporting Engine**: `HandoffReport` container in `code/aiosh-rust/aiosh-core/src/handoff.rs` with `total_handoffs`, `active_handoffs`, `completed_handoffs` distributions.
- **Arithmetic Invariant Enforcement**: `validate_handoff_report` guaranteeing `active + completed == total`.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H8` observability metrics and report validation.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U17 passing in isolation.
- **Documentation**: Updated `docs/README.md` with observability metrics structure.
- **Ledger Pointer**: Advances to **T-00991** (`Agent Handoff Protocol / documentation: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Security Policy CLOSED 10/10 (T-00971..T-00980)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / security policy`:
- **Role-Based Authorization Gate**: `can_agent_act` and `verify_handoff_authorization` methods in `code/aiosh-rust/aiosh-core/src/handoff.rs` restricting control actions.
- **Fail-Closed Execution**: Rejection with `PermissionDenied` on any unauthorized caller interception.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H7` security policy and actor authorization matrix.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U15 passing in isolation.
- **Documentation**: Updated `docs/README.md` with security authorization details.
- **Ledger Pointer**: Advances to **T-00981** (`Agent Handoff Protocol / observability: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Automated Tests CLOSED 10/10 (T-00961..T-00970)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / automated tests`:
- **State Matrix & Edge-Case Harness**: Comprehensive tests in `code/aiosh-rust/aiosh-core/src/handoff_service.rs` verifying rejection paths, cancellations, terminal state immutability, and 50+ concurrent requests.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H6` automated edge cases, state matrix, and batch fuzzing.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U13 passing in isolation.
- **Documentation**: Updated `docs/README.md` with automated test details.
- **Ledger Pointer**: Advances to **T-00971** (`Agent Handoff Protocol / security policy: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Configuration CLOSED 10/10 (T-00951..T-00960)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / configuration`:
- **Configuration Engine (`aiosh-core::handoff_config`)**: `HandoffConfig` container managing storage caps, defaults, and TTL expiration settings.
- **Config-Aware Store Integration**: `HandoffStore::load_from_path_with_config` and `load_or_recover_with_config` enforcing configured thresholds.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H5` configuration validation, bounds checking, and disk roundtripping.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U11 passing in isolation.
- **Documentation**: Updated `docs/README.md` with configuration parameters and environment variables.
- **Ledger Pointer**: Advances to **T-00961** (`Agent Handoff Protocol / automated tests: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol MCP/API Surface CLOSED 10/10 (T-00941..T-00950)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / MCP/API surface`:
- **MCP Tool Endpoints (`aiosh-mcp`)**: `aios.handoff.list`, `show`, `initiate`, `accept`, `reject`, `complete`, `cancel` exposed via JSON-RPC 2.0.
- **PEP & Audit Integration**: Handled via `dispatch::recorded_call` ensuring strict PEP evaluation and SQLite audit trail emission.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H4` MCP surface tools and flow.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U09 passing in isolation.
- **Documentation**: Updated `docs/README.md` with MCP tool schemas and JSON-RPC examples.
- **Ledger Pointer**: Advances to **T-00951** (`Agent Handoff Protocol / configuration: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol CLI Surface CLOSED 10/10 (T-00931..T-00940)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / CLI surface`:
- **CLI Commands (`aiosh-cli`)**: `aiosh handoff [list|show|initiate|accept|reject|complete|cancel]` with full parameter validation and exit codes.
- **Audit Compliance**: Synchronous audit row emission via `classify_and_emit` on every state change.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H3` CLI surface subcommands and flow.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U07 passing in isolation.
- **Documentation**: Updated `docs/README.md` with CLI commands and examples.
- **Ledger Pointer**: Advances to **T-00941** (`Agent Handoff Protocol / MCP/API surface: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Core Service CLOSED 10/10 (T-00921..T-00930)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / core service`:
- **State Store (`aiosh-core::handoff_service`)**: `HandoffStore` with state transitions (`accept`, `reject`, `complete`, `cancel`), deduplication, active queue filtering, and report generation.
- **Persistence & Recovery**: Atomic temporary file write (`.tmp`) and fail-safe corruption recovery (`load_or_recover`).
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H2` core service store lifecycle, transitions & persistence.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U05 passing in isolation.
- **Documentation**: Updated `docs/README.md` with core service store documentation.
- **Ledger Pointer**: Advances to **T-00931** (`Agent Handoff Protocol / CLI surface: Research`).

### 2026-08-31 — MILESTONE: Agent Handoff Protocol Data Model CLOSED 10/10 (T-00911..T-00920)

Complete implementation, governance, verification, and hardening for `Agent Handoff Protocol / data model`:
- **Core Primitives (`aiosh-core::handoff`)**: `HandoffRecord` container (`HND-<hash>`), `HandoffReport`, status & priority enums, deterministic SHA-256 fingerprinting.
- **Invariant Validation**: `validate_handoff_record` and `validate_handoff_report` structural checkers.
- **Test Runner Matrix (`tools/test_handoff_suites.py`)**: Criterion `H1` data model integrity and signature determinism.
- **Unit Suite (`tools/test_handoff_unit.py`)**: Behavioral assertions U01..U03 passing in isolation.
- **Documentation**: New section `## Agent Handoff Protocol (T-00911..T-01000)` in `docs/README.md`.
- **Ledger Pointer**: Advances to **T-00921** (`Agent Handoff Protocol / core service: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Recovery & Validation CLOSED 10/10 (T-00901..T-00910) — EPIC CLOSED (100/100)

Complete implementation, governance, verification, and hardening for `Regression Triage / recovery & validation`:
- **Validation Engine (`aiosh-core::triage`)**: `validate_triage_record` structural checks (ID, SHA-256 fingerprint, non-empty fields, occurrence bounds).
- **Resilient Recovery (`aiosh-core::triage_service`)**: `TriageStore::load_or_recover` gracefully handling corrupted or invalid store files with diagnostic warnings.
- **Criteria T1..T8 Matrix (`tools/test_triage_suites.py`)**: Full coverage across data model, core service, CLI, MCP tools, config, E2E lifecycle, observability, and recovery resilience.
- **Unit Suite Extensions (`tools/test_triage_unit.py`)**: Behavioral assertions U01..U09 passing in isolation.
- **EPIC COMPLETE**: Regression Triage Epic (`T-00811..T-00910`) **100/100 tasks CLOSED**.
- **Ledger Pointer**: Advances to **T-00911** (`Agent Handoff Protocol / data model: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Documentation CLOSED 10/10 (T-00891..T-00900)

Complete implementation, governance, verification, and hardening for `Regression Triage / documentation`:
- **Comprehensive Reference (`docs/README.md`)**: Complete specifications for data models, core service store, CLI subcommands, MCP JSON-RPC tools, `TriageConfig`, automated testing, security invariants, and observability.
- **Structural Integrity (C1..C6)**: Validated using `tools/check_task_docs.py`.
- **Ledger Pointer**: Advances to **T-00901** (`recovery & validation: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Observability CLOSED 10/10 (T-00881..T-00890)

Complete implementation, governance, verification, and hardening for `Regression Triage / observability`:
- **Observability Metrics (`aiosh-core::triage`)**: `status_counts()`, `severity_counts()`, and standardized `summary_line()` diagnostics on `TriageReport`.
- **Test Runner Criterion T7 (`tools/test_triage_suites.py`)**: Validates diagnostic calculations and lifecycle reporting.
- **Unit Suite Extensions (`tools/test_triage_unit.py`)**: Extended U01..U08 behavioral assertions.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00891** (`documentation: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Security Policy CLOSED 10/10 (T-00871..T-00880)

Complete implementation, governance, verification, and hardening for `Regression Triage / security policy`:
- **Security Policy Invariants (`SECURITY.md`)**: Formally defined prohibitions against falsifying or bypassing regression triage records; mandated immutable SQLite WAL audit emission.
- **Evidence Linking**: Linked security review `docs/tasks/evidence/T-00877-security.md` into `SECURITY.md` § Security Knowledge Index.
- **Automated OpenSSF Scorecard Compliance**: Verified criteria S1..S5 in `tools/check_security_policy.py`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00881** (`observability: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Automated Tests CLOSED 10/10 (T-00861..T-00870)

Complete implementation, governance, verification, and hardening for `Regression Triage / automated tests`:
- **Automated Test Matrix (`tools/test_triage_suites.py`)**: Full coverage of criteria `T1..T6` (data model, core store, CLI, MCP tools, config, and E2E recurrence lifecycle).
- **Behavioral Unit Test Suite (`tools/test_triage_unit.py`)**: Validates test runner functions, execution harness, and exit code isolation.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00871** (`security policy: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Configuration CLOSED 10/10 (T-00851..T-00860)

Complete implementation, governance, verification, and hardening for `Regression Triage / configuration`:
- **Configuration Module (`aiosh-core::triage_config`)**: `TriageConfig`, `should_ingest_suite`, parameter boundary checks, and size-capped persistence.
- **Service Integration (`aiosh-core::triage_service`)**: `load_from_path_with_config` and `ingest_ci_summary_with_config`.
- **CLI Integration (`aiosh-cli::main`)**: `--config <path>` flag and `$AIOS_TRIAGE_CONFIG` environment override.
- **Test Runner (`tools/test_triage_suites.py`)**: Criterion `T5` validating configuration schema, validation rules, and suite filters.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00861** (`automated tests: Research`).

### 2026-08-31 — MILESTONE: Regression Triage MCP/API Surface CLOSED 10/10 (T-00841..T-00850)

Complete implementation, governance, verification, and hardening for `Regression Triage / MCP/API surface`:
- **MCP JSON-RPC Tools (`aiosh-mcp::Server`)**: `aios.triage.list`, `aios.triage.show`, `aios.triage.record`, `aios.triage.resolve`, `aios.triage.check` with parameter validation and SQLite WAL audit logging.
- **Test Runner (`tools/test_triage_suites.py`)**: Criterion `T4` validating tool manifest, execution, error states, and resolution cycle.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00851** (`Fingerprinting & dedup: Research`).

### 2026-08-31 — MILESTONE: Regression Triage CLI Surface CLOSED 10/10 (T-00831..T-00840)

Complete implementation, governance, verification, and hardening for `Regression Triage / CLI surface`:
- **CLI Subcommands (`aiosh-cli::main`)**: `aiosh triage` supporting `list`, `show`, `record`, `resolve`, `ingest`, and `check` with JSON output and SQLite WAL audit trail logging.
- **Test Runner (`tools/test_triage_suites.py`)**: Criterion `T3` validating CLI commands, parameter handling, and exit codes.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00841** (`MCP/API: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Core Service CLOSED 10/10 (T-00821..T-00830)

Complete implementation, governance, verification, and hardening for `Regression Triage / core service`:
- **Core Service (`aiosh-core::triage_service`)**: `TriageStore`, `ingest_ci_summary`, failure deduplication, resolution transitions, reopening upon regression recurrence, and size-capped disk persistence.
- **Test Runner (`tools/test_triage_suites.py`)**: Criterion `T2` validating store persistence, status lifecycle, and CI summary ingestion.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00831** (`CLI: Research`).

### 2026-08-31 — MILESTONE: Regression Triage Data Model CLOSED 10/10 (T-00811..T-00820)

Complete implementation, governance, verification, and hardening for `Regression Triage / data model`:
- **Core Types (`aiosh-core::triage`)**: `TriageStatus`, `TriageSeverity`, `TriageRecord`, `TriageReport`, `validate_triage_report`, and deterministic `compute_failure_signature`.
- **Test Runner (`tools/test_triage_suites.py`)**: Criterion `T1` verifying data model integrity and failure fingerprinting.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00821** (`core service: Research`).

### 2026-08-31 — GRAND MILESTONE: Secrets & Access Hygiene Subsystem CLOSED (100/100 tasks T-00711..T-00810)

Complete implementation, governance, verification, and hardening for all sub-epics of `Secrets & Access Hygiene`:
- **Recovery & Validation (`aiosh-core::secrets`)**: `validate_secret_report` enforcing structural invariants, fault-tolerant scanning, and documented remediation protocols.
- **Criteria K1..K9 Suite (`tools/test_secrets_suites.py`)**: Full coverage of data models, scanners, CLI, MCP tools, config, observability, and validation.
- **Documentation & Invariants**: Updated `docs/README.md` and `SECURITY.md` passing all C1..C6 and S1..S5 invariants.
- **Ledger Pointer**: Advances to **T-00811**.

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Documentation CLOSED 10/10 (T-00791..T-00800)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / documentation`:
- **Subsystem Reference (`docs/README.md`)**: Comprehensive documentation covering data models, scanning engines, CLI usage, MCP JSON-RPC schemas, and configuration.
- **Documentation Invariant Health**: 100% compliant with criteria C1..C6 in `tools/check_task_docs.py`.
- **Ledger Pointer**: Advances to **T-00801** (`recovery & validation: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Observability CLOSED 10/10 (T-00781..T-00790)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / observability`:
- **Observability Methods (`aiosh-core::secrets`)**: `SecretScanReport::severity_counts()` and `SecretScanReport::summary_line()` providing structured severity breakdowns and human-readable diagnostics.
- **Automated Test Criteria K8**: Extended `tools/test_secrets_suites.py` validating metrics integrity and telemetry formatting.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00791** (`documentation: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Security Policy CLOSED 10/10 (T-00771..T-00780)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / security policy`:
- **Security Policy Integration (`SECURITY.md`)**: Formalized vulnerability criteria prohibiting plaintext credential emission and scanner bypass; linked security review `docs/tasks/evidence/T-00777-security.md`.
- **Automated OpenSSF Scorecard Compliance**: Verified criteria S1..S5 in `tools/check_security_policy.py`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00781** (`observability: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Automated Tests CLOSED 10/10 (T-00761..T-00770)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / automated tests`:
- **Automated Suite Runner (`tools/test_secrets_suites.py`)**: Complete validation of criteria K1..K7 covering data model, private keys, API tokens, configuration credentials, CLI commands, MCP tools, and `SecretsConfig`.
- **Hardening & Isolation**: Enforced 120s execution timeouts, isolated ephemeral temp directories, and robust error diagnostics.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00771** (`security policy: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Configuration CLOSED 10/10 (T-00751..T-00760)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / configuration`:
- **Configuration Engine (`aiosh-core::secrets_config`)**: Shipped `SecretsConfig` with versioning, bounded file/line limits, ignored directories, and allowlist patterns with strict schema validation.
- **Default Baseline & Precedence**: Created `docs/secrets_config.json` with precedence order `--config` $\to$ `AIOS_SECRETS_CONFIG` $\to$ `docs/secrets_config.json` $\to$ `SecretsConfig::default()`.
- **Automated Tests & Invariants**: Unit test coverage in `secrets_config::tests` (3/3 PASS), CLI integration in `aiosh-cli::task_cli_tests` (16/16 PASS), standalone runner `tools/test_secrets_suites.py` validating criteria `K1..K7`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00761** (`automated tests: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene MCP/API Surface CLOSED 10/10 (T-00741..T-00750)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / MCP/API surface`:
- **MCP Server Tools (`aiosh-mcp`)**: Shipped `aios.secrets.scan` and `aios.secrets.check` with JSON-RPC 2.0 schemas, full workspace/file scanning, and safe redaction guarantees.
- **Dispatch Gating & Auditing**: Calls route via `dispatch::recorded_call` generating honest SQLite WAL audit rows.
- **Automated Tests & Invariants**: Unit test coverage in `aiosh_mcp::tests` passing 4/4, standalone runner `tools/test_secrets_suites.py` validating criteria `K1..K6`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00751** (`configuration: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene CLI Surface CLOSED 10/10 (T-00731..T-00740)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / CLI surface`:
- **CLI Subcommand Surface (`aiosh-cli`)**: Shipped `aiosh secrets scan` and `aiosh secrets check` with full `--repo`, `--file`, `--max-bytes`, and `--json` support.
- **Prose & Machine-Readable Output**: Clean finding cards with redacted snippets, cryptographic fingerprints, and severity breakdown counters.
- **Automated Tests & Invariants**: Unit test coverage in `aiosh-cli::task_cli_tests` passing 16/16, standalone suite runner `tools/test_secrets_suites.py` validating criteria `K1..K5`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00741** (`MCP server surface: Research`).

### 2026-08-31 — MILESTONE: Secrets & Access Hygiene Core Service CLOSED 10/10 (T-00721..T-00730)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / core service`:
- **Core Service Operations (`aiosh-core::secrets_service`)**: `scan_file_for_secrets` (SEC-001 private keys, SEC-002 AWS access keys, SEC-003 GitHub PATs, SEC-004 generic API keys, SEC-005 configuration passwords) with null-byte binary filtering, line length limits (4096 bytes), and 16 MiB size bounds.
- **Workspace Ingestion (`scan_workspace_for_secrets`)**: Full directory recursion skipping VCS/build directories (`.git`, `target`, `node_modules`, `.venv`, `dist`) with aggregated `SecretScanReport` outputs.
- **Unit & Suite Testing**: Unit test coverage in `secrets_service::tests` passing 7/7, and test runner `tools/test_secrets_suites.py` validating criteria `K1..K4`.
- **Documentation & Invariants**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00731** (`CLI surface: Research`).

### 2026-08-30 — MILESTONE: Secrets & Access Hygiene Data Model CLOSED 10/10 (T-00711..T-00720)

Complete implementation, governance, verification, and hardening for `Secrets & Access Hygiene / data model`:
- **Data Model Core (`aiosh-core::secrets`)**: `SecretSeverity` (`Critical`, `High`, `Medium`, `Low`, `Info`), `SecretPatternKind` (`PrivateKey`, `ApiToken`, `AwsCredentials`, `PasswordInConfig`, `HighEntropyGeneric`), `SecretFinding`, and `SecretScanReport`.
- **Safe Boundary Redaction (`redact_secret_value`)**: Preserves 4 prefix / 4 suffix characters for strings $\ge 12$ chars with `****` masking and full multi-byte Unicode boundary handling.
- **Invariant Validation (`validate_secret_report`)**: Enforces arithmetic consistency across severity totals and array lengths.
- **Unit & Baseline Testing**: Unit test coverage in `secrets::tests` passing 5/5, and standalone test suite runner `tools/test_secrets_suites.py` validating criteria `K1`.
- **Documentation & Verification**: Updated `docs/README.md` passing all C1..C6 structural invariants.
- **Ledger Pointer**: Advances to **T-00721** (`service & ingest: Research`).

### 2026-08-30 — GRAND MILESTONE: Repository Health CLOSED 100/100 (T-00611..T-00710)

Complete architecture, data modeling, APIs, CLI, MCP tools, automated invariant checkers, security policies, observability diagnostics, documentation, and recovery routines for the entire **Repository Health Diagnostics** component (100/100 tasks complete):
- **Data Model & Schema (T-00611..T-00620)**: `RepoHealthReport`, `RepoHealthCheck`, `HealthStatus`, and `HealthCategory` in `aiosh-core::repo_health`.
- **Service & Ingest Layer (T-00621..T-00630)**: Built `aiosh-core::repo_health_service` supporting git tree hygiene, 16 MiB file bounds scanning, and security governance audits.
- **CLI Subcommand Surface (T-00631..T-00640)**: Delivered `aiosh repo health` and `aiosh repo check` with dual prose and `--json` rendering.
- **MCP Tool Integration (T-00641..T-00650)**: Exposed `aios.repo.health` and `aios.repo.check` with JSON-RPC 2.0 schemas.
- **Configuration & Defaults (T-00651..T-00660)**: Added `RepoHealthConfig` with Twelve-Factor env resolution and 64 KiB configuration security cap.
- **Automated Tests & Invariants (T-00661..T-00670)**: Deployed `tools/test_repo_health_suites.py` asserting criteria `H1..H7`.
- **Security Policy & Governance (T-00671..T-00680)**: Enforced OpenSSF criteria compliance, read-only guarantees, and immutable audit logs.
- **Observability & Diagnostics (T-00681..T-00690)**: Added sub-millisecond `duration_ms` per-check and aggregate telemetry counters.
- **Documentation & Formatter (T-00691..T-00700)**: Implemented `format_repo_health_summary` with 50-item detail clamping (`tools/check_task_docs.py` C1..C6).
- **Recovery & Validation (T-00701..T-00710)**: Implemented `recover_default_repo_health_config`, `reconstruct_repo_health_report`, `validate_repo_health_report`, and `reconcile_repo_health` ensuring zero-downtime diagnostic recovery.
- **Ledger Pointer**: Advances to **T-00711**.

### 2026-08-30 — MILESTONE: Repository Health Documentation CLOSED 10/10 (T-00691..T-00700)

Complete implementation, governance, verification, and hardening for `Repository Health / documentation`:
- **Human-Readable Formatter (`format_repo_health_summary`)**: Renders clean console and log summaries of `RepoHealthReport` with per-check elapsed timings, status indicators (`[Pass]`, `[Warn]`, `[Fail]`, `[Skip]`), and aggregate counters.
- **Defensive Detail Clamping**: Standardized 50-item detail clamping with explicit truncated item notification (`... (<N> additional items truncated)`).
- **Unit & Smoke Testing**: Unit test coverage in `repo_health_service::tests` passing 9/9, along with CLI and MCP smoke suites.
- **Verification & Documentation**: Verified across all test suites (`test_repo_health_suites.py` H1..H7, `check_security_policy.py` S1..S5, `check_task_docs.py` C1..C6, `check_evidence.py` E1..E4).
- **Ledger Pointer**: Advances to **T-00701** (`recovery & validation: Research`).

### 2026-08-29 — MILESTONE: Repository Health Observability CLOSED 10/10 (T-00681..T-00690)

Complete implementation, governance, verification, and hardening for `Repository Health / observability`:
- **Timing & Counter Observability (`RepoHealthReport`, `RepoHealthCheck`)**: Added `duration_ms` timing per individual check and summary level, along with aggregate status counters (`total_checks`, `passed_checks`, `warn_checks`, `failed_checks`, `skipped_checks`).
- **CLI and MCP Diagnostics**: Structured health inspection surfaced via CLI (`aiosh repo health [--json]`) and MCP (`aios.repo.health`).
- **Hardening & Defensive Limits**: Enforced 50-item clamping on untracked git details, ignored heavy build folders (`.git`, `target`, `node_modules`, `.venv`), and safe argument vectorization for subprocesses.
- **Verification & Documentation**: Verified across all test suites (`test_repo_health_suites.py` H1..H7, `check_security_policy.py` S1..S5, `check_task_docs.py` C1..C6, `check_evidence.py` E1..E4).
- **Ledger Pointer**: Advances to **T-00691** (`documentation: Research`).

### 2026-08-29 — GRAND MILESTONE: Evidence & Audit Trail CLOSED 100/100 (T-00511..T-00610)

Complete architecture, data modeling, APIs, CLI, MCP tools, automated invariant checkers, security policies, observability diagnostics, documentation, and recovery routines for the entire **Evidence & Audit Trail** component (100/100 tasks complete):
- **Data Model & Schema (T-00511..T-00520)**: Formally specified `TaskEvidenceManifest`, `EvidenceRecord`, `EvidenceStep`, and SQLite WAL audit records with deterministic SHA-256 attestation.
- **Service & Ingest Layer (T-00521..T-00530)**: Built `aiosh-core::evidence_service` supporting atomic manifest creation, record append, SHA-256 verification, and strict path containment.
- **CLI Subcommand Surface (T-00531..T-00540)**: Delivered `aiosh evidence hash`, `aiosh evidence verify`, and `aiosh evidence scan` with dual prose and `--json` rendering.
- **MCP Tool Integration (T-00541..T-00550)**: Exposed `aios.evidence.hash`, `aios.evidence.verify`, and `aios.evidence.scan` with full JSON-RPC 2.0 schemas.
- **Configuration & Defaults (T-00551..T-00560)**: Added `EvidenceConfig` (`config/evidence.config.json`) with environment variable overrides and a 64 KiB configuration security cap.
- **Automated Tests & Invariant Checkers (T-00561..T-00570)**: Deployed `tools/check_evidence.py` asserting criteria `E1` (directory health), `E2` (ledger consistency), `E3` (file bounds & UTF-8), and `E4` (deterministic SHA-256).
- **Security Policy & PEP (T-00571..T-00580)**: Enforced PEP token gating on mutating actions, fail-closed denial policies, and audit trail tamper resistance (`tools/check_security_policy.py` criteria `S1`..`S5`).
- **Observability & Diagnostics (T-00581..T-00590)**: Added `EvidenceTelemetry` and `collect_evidence_telemetry` with 512-byte outcome string clamping (`clamp_str`).
- **Documentation & Formatter (T-00591..T-00600)**: Implemented `format_evidence_summary`, updated `docs/README.md` passing all doc invariants (`tools/check_task_docs.py` `C1`..`C6`).
- **Recovery & Validation (T-00601..T-00610)**: Implemented `recover_default_evidence_config`, `reconstruct_evidence_manifest`, `scan_evidence_directory`, and `reconcile_evidence_manifest` ensuring zero-downtime recovery of task evidence catalogs.
- **Ledger Pointer**: Advances to **T-00611**.

### 2026-08-29 — MILESTONE: Evidence & Audit Trail Observability CLOSED 10/10 (T-00581..T-00590)

Complete implementation, governance, verification, and hardening for `Evidence & Audit Trail / observability`:
- **Observability Data Model (`EvidenceTelemetry`)**: Standardized aggregate metrics schema (`total_records`, `valid_records`, `missing_files_count`, `hash_mismatches_count`, `is_healthy`) derived deterministically via `collect_evidence_telemetry()`.
- **Diagnostic Invariants & Clamping**: Structured audit trail event emission with 512-byte outcome string clamping (`clamp_str`) protecting SQLite WAL from buffer flooding.
- **Unit Testing**: Comprehensive unit tests covering healthy reports, degraded reports, empty boundary states, all-missing states, and JSON serialization roundtrips (`test_collect_evidence_telemetry`).
- **Cross-Substrate Integration & Verification**: Validated full integration across CLI (`aiosh evidence verify --json`), MCP (`aios.evidence.verify`), and Rust integration test harnesses (`test_evidence_e2e.rs`).
- **Documentation & Verification**: Integrated observability documentation into `docs/README.md` passing all structural doc invariants (`tools/check_task_docs.py` C1..C6).
- **Ledger Pointer**: Advances to T-00591 (`documentation: Research`).

### 2026-08-29 — MILESTONE: Evidence & Audit Trail Security Policy CLOSED 10/10 (T-00571..T-00580)

Complete implementation, governance, verification, and hardening for `Evidence & Audit Trail / security policy`:
- **Security Policy Invariants (`SECURITY.md`)**: Formally classified evidence tampering, checksum forgery, and out-of-bounds artifact traversal under reportable vulnerability definitions, verified in CI via `tools/check_security_policy.py` (criteria S1..S5).
- **PEP Enforcement & Authorization (`evidence_service.rs` / `pep.rs`)**: Gated all mutating actions (`aios.evidence.record`, `evidence.record`, `aios.evidence.set`, `evidence.set`) behind verified PEP grant tokens with fail-closed default, while permitting unauthenticated read-only operations (`hash`, `scan`, `verify`).
- **Audit Ring Refusals (ADR-0035 §F-2)**: Structured logging of policy denials appending honest `outcome="refused"` rows to SQLite WAL.
- **Unit & Integration Testing**: Unit test coverage in `evidence_service::tests::test_check_evidence_policy_enforcement` passing with zero regressions across CLI and MCP smoke suites.
- **Documentation & Verification**: Integrated security policy documentation into `docs/README.md` passing all structural doc invariants (`tools/check_task_docs.py` C1..C6).
- **Ledger Pointer**: Advances to T-00581 (`observability: Research`).

### 2026-08-29 — MILESTONE: Evidence & Audit Trail Automated Tests CLOSED 10/10 (T-00561..T-00570)

Complete implementation, governance, verification, and hardening for `Evidence & Audit Trail / automated tests`:
- **Evidence Verification Checker (`tools/check_evidence.py`)**: Asserts criteria `E1` (directory health), `E2` (ledger consistency across sampled completed tasks), `E3` (16 MiB size bounds & valid UTF-8), and `E4` (deterministic SHA-256 calculation).
- **Behavioral Unit Test Suite (`tools/test_check_evidence.py`)**: 15 test cases (U01..U14 + S01) testing positive, negative, boundary, and blindness sensitivity in temporary isolated sandboxes.
- **Cross-Substrate Tests & CI Integration (`tools/ci_suites.py`)**: Registered `evidence_cli_smoke`, `evidence_mcp_smoke`, `evidence_checker`, and `evidence_unit` into the canonical CI runner registry, verified by `tools/test_ci_suites.py` (W1..W7).
- **Rust End-to-End Suite (`test_evidence_e2e.rs`)**: 10-step lifecycle manifest verification, tampering detection, missing file reporting, and query helpers.
- **Documentation & Verification**: Operator manuals and execution examples integrated in `docs/README.md` passing all structural doc invariants (`tools/check_task_docs.py` C1..C6).
- **Ledger Pointer**: Advances to T-00571 (`security policy: Research`).

### 2026-08-28 — MILESTONE: Dependency & Toolchain Pinning Epic CLOSED 90/90 (T-00311..T-00400)

Complete implementation, governance, verification, and hardening for `Dependency & Toolchain Pinning`:
- **Core Manifest & Data Model (T-00311..T-00320)**: `ToolchainManifest` supporting `rust_version`, `python_version`, `node_version`, and `enforce_hashes` with provenance tracing (`to_json_with_sources`).
- **MCP Server Surface (T-00321..T-00330)**: `aios.toolchain.config.get` and `aios.toolchain.check` tools with typed JSON-RPC handlers.
- **CLI Surface (T-00331..T-00340)**: `aiosh toolchain show` and `aiosh toolchain check [--config <path>]` commands.
- **Configuration & Overrides (T-00341..T-00350)**: Multi-source resolution (`$AIOSH_TOOLCHAIN_CONFIG`, `config/toolchain.json`, defaults) bounded by 64KB file caps.
- **Physical Enforcement (T-00351..T-00360)**: Native Rust version probing (`rustc -V`, `python3 -V`, `node -v`) with 15s execution timeouts and child process reap.
- **Automated Tests (T-00361..T-00370)**: Standalone smoke suites (`test_toolchain_cli_smoke.py`, `test_toolchain_mcp_smoke.py`) integrated into CI runner (`tools/ci_suites.py`).
- **Security Policy & PEP (T-00371..T-00380)**: PEP token gating for mutating actions (`aios.toolchain.set`), `SECURITY.md` criteria (S1..S5), and audit WAL persistence.
- **Observability & Diagnostics (T-00381..T-00390)**: `ToolchainTelemetry`, 512-byte string output clamping (`clamp_str`), and `aiosh audit tail` diagnostic access.
- **Documentation (T-00391..T-00400)**: Canonical operator and agent reference manual in `docs/README.md` passing all C1..C6 structural invariants (`tools/check_task_docs.py`).
- **Ledger Pointer**: Advances to T-00401.

### 2026-08-23 — MILESTONE: CI Smoke Orchestration data model CLOSED 10/10 (T-00111..T-00120)

`tools/ci_suites.py` (registry: 19 suites, order-is-contract, timeouts)
+ `tools/ci_run.py` (production orchestrator: per-suite wall-clock
timeouts with process-group kill, bounded log tails, atomic
machine-readable run summary at `$AIOSH_CI_RESULTS`). `ci/run_all_smokes.sh`
is now a delegating shim; the registry is the single source. Hardening:
group-kill verified zero survivors; 5 MiB log → 12 B tail output.
Verified: full CI **19/19 PASS** through the new path (180515 ms) + W-suite
W1..W7 (`docs/tasks/evidence/T-00120-verify.md`). Ledger pointer: T-00121.

### 2026-08-23 — MILESTONE: Recovery & validation component CLOSED 10/10 (T-00101..T-00110)

`task validate` shipped on all four surfaces (Rust CLI, Rust MCP,
Python MCP reference, Python reference CLI): read-only integrity report
comparing live `TASK_STATE.json` against deterministic event-log replay —
drift/seq/pointer checks fatal, evidence existence+orphans warning-only;
report-only by design (rebuild stays sole repair). Hardening closed
security finding F-1 (evidence path confinement); full findings payload is
byte-parity across Rust-MCP and Python-MCP (modulo audit_id). Verified:
full CI **19/19 PASS** + cargo 82 tests
(`docs/tasks/evidence/T-00110-verify.md`). Ledger pointer: T-00111.

### 2026-08-22 — MILESTONE: Task Ledger Control epic CLOSED 10/10 (T-00011..T-00100)

Documentation component shipped and verified: `tools/check_task_docs.py`
(six structural doc-invariants C1..C6, capped reads, root-bounded link
containment), U-suite **20/20** incl. a blindness-sensitivity proof,
both suites permanent in CI. Docs: README §"Documentation invariants"
with live-verified example + limitations. Verified: full CI **19/19
PASS** + cargo 79 tests (`docs/tasks/evidence/T-00100-verify.md`).
All ten components now closed: data model, core service, CLI,
MCP/API, configuration, automated tests, security policy,
observability, documentation, recovery & validation ← next: T-00101.

### 2026-08-22 — Observability sub-epic CLOSED (T-00081..T-00090)

`aios.task {action:"metrics"}` / `aiosh task metrics` shipped and
verified: stable additive-only snapshot `{tasks, audit, config}`,
counts-only disclosure, grant-free read-only, one honest audit row per
call. Tests-first caught two real defects pre-review (wire accepted
task_id on metrics; CLI silently ignored stray operands) — both fixed
and pinned by the new permanent `metrics_smoke` suite (O1–O8).
Discoverability added to the published inputSchema enum on both
substrates; O(1) COUNT(*) hardening; SPEC §8.6 operator docs.
Verified: cargo 79 tests + **CI 17/17 PASS**
(`docs/tasks/evidence/T-00090-verify.md`). Ledger pointer: T-00091.

### 2026-08-22 — Security policy sub-epic CLOSED (T-00071..T-00080)

Root `SECURITY.md` shipped (OpenSSF criteria met; owner-provided
advisory channel; scope from six component reviews; 7d/90d CVD;
rule-pack governance) + permanent `security_policy` CI suite. Review:
no fabrications/secrets; links verified. **CI 16/16 PASS**
(`docs/tasks/evidence/T-00080-verify.md`). Ledger pointer: T-00081.

### 2026-08-22 — Automated tests sub-epic CLOSED (T-00061..T-00070)

New cross-surface matrix suite (`test_ledger_matrix_smoke.py`, M1–M8)
pins what per-surface tests cannot: one-grant-both-servers,
narrow-grant rejection, concurrent-writer lock-busy (bounded), config
propagation, grant expiry fail-closed, block/unblock. Wired into CI →
**15/15 suites PASS**. Suites hardened (subprocess timeouts,
holder kill-safety); suites themselves security-reviewed (no leaks/no
bypass). Design fact encoded: `rebuild` is lock-free by design.
Ledger pointer: T-00071.

### 2026-08-22 — Configuration sub-epic CLOSED (T-00051..T-00060)

Ledger knobs are now operator-configurable via six `AIOSH_LEDGER_*`
env vars (Twelve-Factor-aligned; defaults == shipped constants; loud
named errors; floors + 86400s lock ceiling; python parity). Exposed by
`aiosh task config`; deliberately NOT agent-exposable via MCP (D5).
Security-reviewed (no open bypass); documented SPEC §8.3. Verified:
79 cargo tests + all wire suites + **CI 14/14 PASS**
(`docs/tasks/evidence/T-00060-verify.md`). Ledger pointer: T-00061.

### 2026-08-22 — MCP/API surface sub-epic CLOSED (T-00041..T-00050)

The Python reference server now mirrors the Rust `aios.task` tool
(`aios_task`: 7 actions, one grant valid across BOTH substrates).
P-suite caught a real gating hole pre-review (`rebuild` mis-classified
read-only) — fixed and permanently pinned. Hardening: module caching,
audited loader failures, bool-id rejection. Verified: 77 cargo tests +
U/W/C/P suites + **CI 13/13 PASS**
(`docs/tasks/evidence/T-00050-verify.md`). SPEC §7 L5 RESOLVED, §8.2
added. Ledger pointer: T-00051.

### 2026-08-22 — CLI surface sub-epic CLOSED (T-00031..T-00040)

`aiosh task` unified onto `task_service::TaskCall` (one validation
source with MCP): strict grammar (u64≥1, non-empty note/reason,
4096/16 caps, dash-value rejection, `--` delimiter), per-subcommand
help; evidence-item cap added to core; **non-UTF-8 argv panic
eliminated** (lossy + audited). Security-reviewed (no open bypass);
documented in SPEC §8.1. Verified: 77 cargo tests, U1..U16, W1..W8,
C1..C9, **CI 12/12 PASS** (`docs/tasks/evidence/T-00040-verify.md`).
Ledger pointer: T-00041 (configuration component begins).

### 2026-08-22 — Core service sub-epic CLOSED (T-00021..T-00030)

The agent-facing ledger surface shipped end-to-end: `aios.task` MCP
tool behind classifier→PEP→audit (read-only status/check; grant-gated
mutations), D3 resolver repair, D4 rebuild replay in both substrates,
bounded lock wait, 1 MiB transport cap. Security-reviewed with no open
bypass; documented in `docs/SPEC-TASK-LEDGER.md` §7–§9; verified with
64 cargo tests + U1..U16 + W1..W8 + **CI 11/11 PASS**
(`docs/tasks/evidence/T-00030-verify.md`). Ledger pointer: T-00031
(CLI-surface sub-epic begins).

### 2026-08-22 — Task Ledger Control epic CLOSED (T-00011..T-00020)

The 10-task ledger-control epic is verified complete: data model
researched, specified, implemented in Rust (`aiosh-core/src/ledger.rs`),
wired as `aiosh task …`, audited, security-reviewed, hardened,
documented (`docs/SPEC-TASK-LEDGER.md`), and verified — full baseline
10/10 PASS with captured evidence
(`docs/tasks/evidence/T-00020-verify.md`). Ledger pointer: T-00021.
Known limitations L1–L5 recorded in the spec §7 as decisions-needed
(Rust default path resolution; rebuild-vs-skip pointer rewind; flock is
single-host only; evidence attested not validated; parity smoke covers
done+block flows).

### 2026-08-21 — FULL RUST REWRITE (user directive, SHIPPED)

The shipping stack is now **Rust** (`code/aiosh-rust/`), replacing the
TypeScript CLI + Python MCP server as the ship path. All Sprint 0-3
capabilities were ported and verified:
- **aiosh-core** — canonical JSON/sha256, audit ring (SQLite WAL + hash
  chain), classifier R-01..R-12, PEP grants, retention (rotation + bloom +
  verify --full), pentest wrappers (nmap/nikto/sqlmap/tshark/aircrack-ng),
  Landlock + seccomp sandbox, Ollama/stub agent loop.
- **aiosh-cli** — `aiosh` binary: `status`, `run`, `audit tail/verify/
  rotate/segments/seen`, `grant create/list/revoke`, `pentest`, `classify`,
  `agent`.
- **aiosh-mcp** — stdio JSON-RPC MCP server (initialize / tools/list — 12
  tools / tools/call) with every call routed through the classifier→PEP→
  audit gate.
- **Green:** zero-warning `cargo build`; 45 `cargo test` cases, including a
  port of the Python classifier fixture matrix (SC1..SC10) pinning
  byte-identical behavior with the legacy substrates; end-to-end smoke
  `code/aiosh-rust/ci/rust_smoke.sh` (build + tests + MCP wire contract +
  CLI status) wired into `ci/run_all_smokes.sh` first.
- The legacy TS (`code/aiosh-cli`) and Python (`code/aiosh-mcp`) trees are
  retained as the cross-substrate reference contract, not the ship path.

### Done in Sprint 0 (shipped, pre-Rust)
- MCP server skeleton + FastMCP stdio transport.
- aiosh-cli (TypeScript): `status`, `run`, `agent` (stub), `audit tail/verify`,
  `grant create/list/revoke`.
- Hash-chained append-only SQLite WAL audit ring.
- Cross-substrate canonical-JSON invariant (TS ↔ Python).
- 5-tool MCP manifest: `aios.fs.read`, `aios.process.list`,
  `aios.audit.tail`, `aios.audit.verify`, `aios.pentest.nmap` (stub).

### Done in Sprint 1 (shipped 2026-08-20)
- Real Pillar-A pentest wrapper set — five tools:
  `pentest.nmap`, `pentest.nikto`, `pentest.sqlmap`,
  `pentest.tshark`, `pentest.aircrack-ng`.
- Both surfaces (MCP and CLI) share the audit ring through the same
  canonical-JSON invariant.
- Every pentest tool call writes one chain-extending audit row.
- 5-suite pentest smoke (`tests/test_pentest_smoke.py`) passes:
  S1 no-grant → refused; S2 grant+no-binary → refused-no-binary;
  S3 scope.tools mismatch → refused; S4 scope.paths mismatch → refused;
  S5 chain integrity holds across TS+Python writers.
- CLI bridge `code/aiosh-cli/src/pentest.ts` exposed as
  `aiosh pentest {nmap|nikto|sqlmap|tshark|aircrack-ng} <args> --grant <id>`.
- Cross-substrate canonical-JSON bug fixed (TS now stores args_json in
  canonical form so nested undefined→null placeholders round-trip).

### Done in Sprint 1.5 (shipped 2026-08-20)
- Replaced the key-grep `cFlagsFor()` with a **deterministic rule-pack
  classifier** (`R-01`…`R-12`) in both TS and Python.
- `classify()` returns `{c_flags, rule_ids, evidence, overall_verdict,
  policy_revision}` — every fired rule contributes a stable rule ID,
  confidence, and human-readable evidence the audit row carries
  verbatim.
- `policy_revision` field (`sprint-1.5-rule-pack-v1`) makes classifier
  behavior version-stamped; any rule-pack change requires a bump.
- Cross-language invariant proven: 10/10 SC fixtures produce
  semantically-identical classifications in TS and Python (after
  numeric-format normalization); the 4 module-level lists are
  byte-equal.
- Bug caught and fixed during smoke: TS `equals` predicate was not
  resolving the `$DANGEROUS_BINS` sentinel, causing asymmetric
  `R-05a` firing between the two languages. Would have shipped
  silent refusals-by-default.
- New `aiosh classify <tool> [--target <t>] [--json-args '{...}']`
  CLI surface for user-driven checks.
- Formal spec: `docs/SPEC-CONSTITUTION-CLASSIFIER.md`.
- See `docs/SPRINT-0.md` §9 for full evidence trail.

### Sprint 2 — agent loop (SHIPPED + VERIFIED 2026-08-21)

The agent loop described below is **built and verified green**. The
`task_plan` text claiming "the remaining gap is the agent that calls
them" was stale relative to the tree — the agent already exists.

Shipped in code:
- `code/aiosh-cli/src/agent.ts` — Computer-Use loop
  (Observe → Think → Act → Loop), Ollama-0.22.1 backend with a
deterministic stub fallback, `classify()` preflight per tool call.
- `code/aiosh-mcp/aiosh_mcp/agent_bridge.py` — persistent MCP stdio
  client forwarding `tools/call` to the real `aiosh_mcp.server`.
- `aiosh agent <prompt>` CLI subcommand (Sprint-0 §2 stub now real).
- MCP dispatch gate (`_dispatch.py` + `server.py`) calls `classify()`
  on every tool — the ADR-0035 §D-4 boundary.

Verified 2026-08-21 (all smokes green after installing mcp/fastmcp +
npm deps and fixing a broken `node_modules/.bin/tsc` wrapper):
```
PASS: Sprint 1.5 classifier smoke (SC1..SC10 + cross-language)
PASS: aiosh-mcp smoke (TS↔Python chain, 9 tools)
PASS: aiosh-mcp Sprint 1 pentest smoke
PASS: aiosh run sandbox smoke (fail-open-with-audit)
PASS: aiosh demo smoke (D1 grant+scan · D2 no-grant refusal ·
      D3 classifier-first refusal)
```
Note: `test_demo_smoke.py` D1 reaches "attempted" but the host lacks
the `nmap` binary — the audited `outcome=refused 'nmap binary not on
PATH'` is the correct auditable answer, not a bug.

### Done in Sprint 3 (shipped 2026-08-21)

- **Audit-ring retention policy** (item 1 of the Sprint 3 queue):
  checkpointed segment rotation + per-segment bloom filters,
  implemented identically in both substrates.
  - New `audit_segments` table; rotation archives the oldest live rows
    byte-identically to `$AIOSH_HOME/audit-archive/segment-<id>.jsonl`,
    pins the file sha256, and records `{first/last row id, row_count,
    genesis_prev_hash, head_hash, bloom}`.
  - Rotation is archival, never destruction (P-2/O-4 compliant, RFC 9162
    §4.13 log-retirement pattern); the live chain re-anchors at the
    checkpoint head and the rotation event itself is an `audit.rotate`
    chain row (O-2). Rotation refuses to run on a broken chain.
  - `verify()` is anchor-aware on both substrates; `verify --full`
    replays every archive file (checksum + per-row re-hash + segment
    linkage) before the live walk.
  - `seen(hash)` answers live / maybe (bloom) / archive (exact scan) /
    no — no false negatives.
  - Surfaces: CLI `aiosh audit rotate [--keep N] [--dry-run]`,
    `audit segments`, `audit seen <hash> [--exact]`,
    `audit verify --full`; MCP `aios.audit.rotate` (PEP grant
    required), `aios.audit.segments`, `aios.audit.seen`,
    `aios.audit.verify(full)`.
  - Docs: `docs/research/AIOS-AUDIT-RING-RETENTION-2026-08-21.md`,
    `docs/SPEC-AUDIT-RETENTION.md`, ADR-0036.
  - Verified 2026-08-21: `tests/test_retention_smoke.py` R1–R7 all
    PASS (incl. TS-rotates→Python-verifies cross-substrate proof);
    all Sprint 0/1/1.5/2 smokes remain green.

### Active task (next)
**Sprint 3 — remaining hardening items.** With retention shipped, the
highest-value next steps (see Queued below) are: (2) the **`aiosh demo`
snap test** formalized into the CI suite; and (3) expanding the five
pentest wrappers toward the full Kali / MITRE ATT&CK v19 taxonomy.

### Queued (Sprint 2)
- **Sprint 3 (SHIPPED 2026-08-26): CI Smoke Orchestration documentation (T-00191..T-00200)**.
  - Formally documented the CI Smoke Orchestration architecture, CLI surface, and configuration parameters, completely wrapping up Phase 0 of the master ledger matrix!
- **Sprint 3 (SHIPPED 2026-08-26): CI Smoke Orchestration observability (T-00181..T-00190)**.
  - Standardized CI health metrics via iosh ci metrics action, completing the Phase 0 integration matrix.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration security policy (T-00171..T-00180)**.
  - Documented CI orchestrator vulnerability boundaries and updated the repository knowledge index.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration automated tests (T-00161..T-00170)**.
  - Brought the legacy ci_run.py orchestrator under automated test coverage.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration configuration (T-00151..T-00160)**.
  - Implemented Twelve-Factor environment configurations for CI orchestration bounds and file paths.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration MCP/API surface (T-00141..T-00150)**.
  - Integrated ios.ci into the Rust MCP server routing table.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration CLI surface (T-00131..T-00140)**.
  - Formally verified the CLI surface integration (iosh ci) implemented preemptively during T-00128.
- **Sprint 3 (SHIPPED 2026-08-25): CI Smoke Orchestration core service (T-00121..T-00130)**.
  - Native Rust implementation of iosh ci check, strict JSON artifact validation, 1MB file bounds, and honest audit row emission.
- **Sprint 2 (SHIPPED + verified 2026-08-21):** Ollama-0.22.1 /
  Anthropic-Computer-Use agent loop over MCP, gated by the Sprint-1.5
  classifier. ADR-0035 §D-2 (MCP tools) was already wired; §D-4
  (classifier gate) is enforced both as an agent-loop preflight
  (`agent.ts`) and at the MCP dispatch boundary (`_dispatch.py`).
- `aiosh demo` end-to-end scripted engagement (snap test).
- **Landlock + seccomp-bpf wrapper around `aiosh run`** —
  shipped in §11 (sandbox.py + cli.ts wiring + 3-scenario smoke).
  Sprint-2 gap closed; remaining env-dependent work is hardening
  the host kernel (Landlock ≥ 5.13 + accepting new seccomp filters),
  not the sandbox code.
- **Research-note tying the shipped rule-pack classifier to the
  neuron / Dynamic Neural Topology substrate:** shipped as
  `docs/research/AIOS-CLASSIFIER-PRIMITIVE-AND-NEURAL-SUBSTRATE-2026-08-20.md`
  (8 sections, 14 sub-headings). The classifier is *deliberately
  separate* from the cognition engine — see the note's
  three-way split: deterministic safety boundary / pluggable agent
  / preserved-but-unshipped neuron substrate.
- **Audit-ring retention policy (rotation / bloom filter)** — SHIPPED
  2026-08-21 in Sprint 3 item 1. See `### Done in Sprint 3` below.
- Expand pentest set to the full Kali/Parrot tool taxonomy across
  MITRE ATT&CK v19 categories.
- Rule-pack expansion beyond `R-12` as new tools / new attack
  categories land (version-stamped via `policy_revision`).

### Research done (2026-08-21)

- **Four open research gaps closed** in
  `docs/research/AIOS-RESEARCH-GAPS-2026-08-21.md` (all anchors fetched
  live 2026-08-21): Kali/MITRE ATT&CK v19.2 taxonomy → 9 proposed new
  wrappers + namespace rule; on-device inference (llama.cpp `ggml-org`,
  OpenAI-compatible `llama serve`, Ollama local/cloud); AI ↔ desktop hook
  (KWin 6 scripting + wlr virtual input + AT-SPI2, semantic-first
  `gui.*` set); prompt-injection defense for MCP *outputs* (R-11 covers
  args only; propose tagged `scan_output_for_pi`). Each carries a
  "Decisions needed" block → becomes ledger tasks per the no-skip law.

## Immediate actions for the next agent session

1. Read `docs/SPRINT-0.md` (Sprint 0 + Sprint 1 sections) — full
   shipped contract. Sprint 3 retention is documented in
   `docs/SPEC-AUDIT-RETENTION.md` + ADR-0036.
2. Read ADR-0035 (S-rank agent architecture) and Pillar-C clauses,
   plus ADR-0036 (audit-ring retention).
3. Pick a task — retention is SHIPPED (2026-08-21, verified green).
   Next queue items: formalize the `aiosh demo` snap test into a CI
   suite, then expand the five pentest wrappers toward the full
   Kali / MITRE ATT&CK v19 taxonomy.
4. Confirm baseline is green before starting (verified 2026-08-21):
   the **Rust** stack is the primary surface —
   `bash code/aiosh-rust/ci/rust_smoke.sh` (build + 45 tests + MCP wire
   contract + CLI status). The legacy suites
   (`python code/aiosh-mcp/tests/test_demo_smoke.py` AND
   `python code/aiosh-mcp/tests/test_smoke.py` AND
   `python code/aiosh-mcp/tests/test_pentest_smoke.py` AND
   `python code/aiosh-mcp/tests/test_retention_smoke.py` AND
   `bash code/aiosh-cli/tests/smoke.sh`) still run to pin the
   cross-substrate invariant.
5. Do not start implementation before research (see
   `mostimportanAIfolder/RESEARCH_EXECUTION_PROTOCOL.md`).

## Constraints

- Do not discard unrelated user changes.
- Do not change kernel code yet; v2 ships on Linux hosts, not the
  microkernel.
- Use repository evidence. ADRs and shipped code (with tests) are
  the source of truth for "what's done".
- Preserve unresolved/deferred tasks explicitly rather than silently
  deleting them.
- Cross-substrate canonical-JSON invariant MUST hold — the Rust
  `canonical()` (sorted keys, no whitespace, floats with `.0`) produces
  byte-identical hash chains with the legacy TS/Python substrates, and
  `code/aiosh-rust/aiosh-core/src/canonical.rs` documents the shared
  contract.

## Errors Encountered (Sprint 0/1)

| Error                                              | Cause                                       | Resolution                                                              |
|---------------------------------------------------|---------------------------------------------|-------------------------------------------------------------------------|
| `read_files` tool rejecting valid string              array | Tool-runtime parameter serialization quirk | Use `cat` via `run_terminal_command` for batch reads                      |
| Chain hash mismatch Sprint 1 row 1                | TS wrote args_json with stripped undefined keys, but canonicalJson hashed with them as null. | TS now writes args_json in canonical form too (`code/aiosh-cli/src/audit.ts`) |
| Sprint 0 cross-process tool-count equality        | Sprint 1 added 4 tools; the equality `actual != expected` broke | Relaxed to subset check: `expected <= actual`                            |
| `outcome: string` not assignable to OutcomeKind | `outcome` was a 3-valued narrow union       | Compute literal `"ok"|"refused"|"error"` and assign                       |
| `pentest.aircrack-ng` audit name with hyphen    | Python identifier can't carry hyphen        | Function named `aios_pentest_aircrack_ng`; audit tool hardcoded literal  |
| `spawned Bash Exit 1` on npx tsc from wrong dir  | shell cwd reset between commands            | `cd /c…and` chains each invocation                                       |

## Current decision

**Active track: Sprint 3 — remaining hardening items. Item 1
(audit-ring retention) SHIPPED 2026-08-21.**

Sprint 1 (pentest wrappers + cross-language audit), Sprint 1.5 (rule-pack
classifier + spec), Sprint 1.5b (Landlock/seccomp sandbox), Sprint 2
(Ollama/Anthropic agent loop over MCP, classifier-gated), and Sprint 3
item 1 (checkpointed segment rotation + bloom filter retention, ADR-0036)
are all shipped and verified green (2026-08-21).

**Next queue items:** (2) formalize `aiosh demo` snap test into CI
suite; (3) expand five pentest wrappers toward full Kali / MITRE
ATT&CK v19 taxonomy.

**Task-DB caveat:** `TASK_DATABASE.json` is a NON-authoritative,
graph-derived reconstruction (`authoritative: false`,
`provenance: graph-derived-recovery`) — its "89/89 COMPLETED" is an
artifact, not a real status. Source of truth for "what's done" is
repository evidence: ADRs + shipped code with green smokes (per
README *How to keep going*). The live v2/Sprint plan is this file.
