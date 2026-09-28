# Task Evidence: T-02341 (Audit Chain Extensions / configuration: Research)

## 1. Metadata
- **Task ID:** `T-02341`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Configuration Subsystem Research
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 5: Configuration (1/10) — Research

---

## 2. Research Scope & Prior Art Analysis

### 2.1 Existing Configuration Architecture in AIOS
In `code/aiosh-rust/aiosh-core`, configuration subsystems (such as `PepGrantConfig`, `PepConfig`, `KernelModuleConfig`, and `NetworkConfig`) adhere to a unified, fail-safe pattern:
1. **Strong Typing & Serde Serialization**: Standard JSON representation with explicit schema validation (`validate(&self)`).
2. **Bounds Enforcement**: Strict limits on numeric options (e.g., query limits, depth bounds, key capacities) to prevent resource starvation or DoS.
3. **Layered Resolution Order**: Default values < File-based JSON configuration (`from_file`) < Environment variables (`from_env` / `AIOS_AUDIT_*`) < Explicit runtime overrides.
4. **Hermetic File Ingestion Bounds**: Enforces `MAX_CONFIG_BYTES` (64 KiB) read cap on configuration files to prevent memory exhaustion from oversized or corrupted files.

### 2.2 Facts vs. Assumptions

| Item | Status | Details |
| :--- | :---: | :--- |
| **Fact** | Confirmed | `AuditChainService` currently hardcodes bounds (`MAX_QUERY_LIMIT = 1000`, `MAX_LINEAGE_DEPTH = 64`). |
| **Fact** | Confirmed | Operators and deployment environments need configurable database paths, default pagination sizes, and lineage depth bounds. |
| **Fact** | Confirmed | SQLite database connection string can be in-memory (`:memory:`) or file-backed (`.aios/audit.db`). |
| **Assumption** | Validated | A dedicated `AuditChainConfig` struct loaded via `from_json`, `from_file`, and `from_env` will provide seamless parity across CLI, MCP, and Core engines. |

### 2.3 Decisions Needed Before Implementation
1. **Configuration Key Structure**:
   - `version: String` (schema version, e.g. `"1.0.0"`)
   - `db_path: PathBuf` (default: `PathBuf::from(".aios/audit.db")`)
   - `max_query_limit: usize` (default: 50, maximum: 1,000)
   - `default_lineage_depth: usize` (default: 16, maximum: 64)
   - `max_causal_links: usize` (default: 16, maximum: 32)
   - `max_extension_bytes: usize` (default: 65,536 = 64 KiB)
   - `verify_signatures_on_read: bool` (default: true)
   - `strict_provenance: bool` (default: false)
2. **Environment Variable Overrides**:
   - `AIOS_AUDIT_CONFIG_PATH`: path to explicit config JSON file
   - `AIOS_AUDIT_DB_PATH`: override SQLite DB path
   - `AIOS_AUDIT_MAX_QUERY_LIMIT`: override query limit clamp
   - `AIOS_AUDIT_LINEAGE_DEPTH`: override ancestry depth clamp

---

## 3. Acceptance Confirmation
- [x] Authoritative facts, constraints, and prior art documented.
- [x] Zero code changes performed during Research phase.
- [x] Technical decisions and bounds specified for upcoming tasks.
