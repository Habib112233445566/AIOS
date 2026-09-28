# Task Evidence: T-02342 (Audit Chain Extensions / configuration: Specification)

## 1. Metadata
- **Task ID:** `T-02342`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Configuration Specification
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 5: Configuration (2/10) — Specification

---

## 2. Technical Specification

### 2.1 Data Structures (`AuditChainConfig`)

```rust
pub const DEFAULT_AUDIT_DB_PATH: &str = ".aios/audit.db";
pub const DEFAULT_MAX_QUERY_LIMIT: usize = 50;
pub const MAX_PERMISSIBLE_QUERY_LIMIT: usize = 1000;
pub const DEFAULT_LINEAGE_DEPTH: usize = 16;
pub const MAX_LINEAGE_DEPTH_BOUND: usize = 64;
pub const DEFAULT_MAX_CAUSAL_LINKS: usize = 16;
pub const MAX_PERMISSIBLE_CAUSAL_LINKS: usize = 32;
pub const DEFAULT_MAX_EXTENSIONS_BYTES: usize = 65536; // 64 KiB
pub const MAX_CONFIG_FILE_BYTES: u64 = 64 * 1024;      // 64 KiB

pub const AUDITCONF_ERR_IO: &str = "AUDITCONF_ERR_IO";
pub const AUDITCONF_ERR_PARSE: &str = "AUDITCONF_ERR_PARSE";
pub const AUDITCONF_ERR_VALIDATION: &str = "AUDITCONF_ERR_VALIDATION";
pub const AUDITCONF_ERR_BOUNDS: &str = "AUDITCONF_ERR_BOUNDS";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditChainConfig {
    pub version: String,
    pub db_path: PathBuf,
    pub max_query_limit: usize,
    pub default_lineage_depth: usize,
    pub max_causal_links: usize,
    pub max_extensions_bytes: usize,
    pub verify_signatures_on_read: bool,
    pub strict_provenance: bool,
}
```

### 2.2 Contract & Method Invariants

1. **`AuditChainConfig::default()`**:
   - `version`: `"1.0.0"`
   - `db_path`: `PathBuf::from(".aios/audit.db")`
   - `max_query_limit`: `50`
   - `default_lineage_depth`: `16`
   - `max_causal_links`: `16`
   - `max_extensions_bytes`: `65536`
   - `verify_signatures_on_read`: `true`
   - `strict_provenance`: `false`

2. **`AuditChainConfig::validate(&self) -> Result<(), String>`**:
   - `version` must be non-empty and start with `"1."`.
   - `max_query_limit` must satisfy $1 \le \text{limit} \le 1000$.
   - `default_lineage_depth` must satisfy $1 \le \text{depth} \le 64$.
   - `max_causal_links` must satisfy $1 \le \text{links} \le 32$.
   - `max_extensions_bytes` must satisfy $1024 \le \text{bytes} \le 1048576$.

3. **`AuditChainConfig::from_json(json_str: &str) -> Result<Self, String>`**:
   - Parses JSON string and validates all bounds. Returns `AUDITCONF_ERR_PARSE` or `AUDITCONF_ERR_BOUNDS` on failure.

4. **`AuditChainConfig::from_file<P: AsRef<Path>>(path: P) -> Result<Self, String>`**:
   - Enforces `MAX_CONFIG_FILE_BYTES = 64 * 1024`. Rejects oversized files with `AUDITCONF_ERR_BOUNDS`.
   - Reads content safely and invokes `from_json`.

5. **`AuditChainConfig::from_env() -> Self`**:
   - Reads `AIOS_AUDIT_CONFIG_PATH` if set, falling back to `default()`.
   - Reads environment overrides `AIOS_AUDIT_DB_PATH`, `AIOS_AUDIT_MAX_QUERY_LIMIT`, `AIOS_AUDIT_LINEAGE_DEPTH`.

6. **`AuditChainConfig::save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String>`**:
   - Serializes configuration to pretty JSON and writes atomically via temporary file replacement.

---

## 3. Acceptance Confirmation
- [x] Inputs, outputs, bounds, and error codes defined.
- [x] Fail-safe fallback and atomic persistence specified.
- [x] Zero API inventions outside standard AIOS patterns.
