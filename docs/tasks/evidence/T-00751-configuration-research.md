# T-00751 — Secrets & Access Hygiene / configuration: Research

## 1. Prior Art & In-Tree Configuration Patterns
- **In-Tree Subsystem Configurations**: Examined `aiosh_core::repo_health_config`, `aiosh_core::doc_index_config`, `aiosh_core::toolchain_config`, and `aiosh_core::evidence_config`.
- **Core Design Principles**:
  - Strongly typed Rust structs with `serde::{Serialize, Deserialize}`.
  - Strict validation on deserialization (`validate()`) preventing out-of-range bounds, buffer overflows, and empty fields.
  - Multi-tier loading priority:
    1. Explicit file path (`from_path`).
    2. Environment variable (`AIOS_SECRETS_CONFIG` via `from_env`).
    3. Default file location (`docs/secrets_config.json`).
    4. Compile-time defaults (`Default::default()`).
  - Strict file read limits (bounded to 64 KiB) to prevent memory denial-of-service on malicious configuration files.

## 2. Facts vs. Assumptions

| Item | Status | Fact / Detail |
|---|---|---|
| Struct Name | Fact | `SecretsConfig` residing in `code/aiosh-rust/aiosh-core/src/secrets_config.rs`. |
| Max Config Size | Fact | 64 KiB read cap (`MAX_CONFIG_BYTES = 64 * 1024`). |
| Default Config Path | Fact | `docs/secrets_config.json` with env fallback `AIOS_SECRETS_CONFIG`. |
| Scan Limits | Fact | Default file size cap 16 MiB (`max_file_bytes`), line length cap 4096 (`max_line_bytes`). |

## 3. Decisions & Contracts Needed
1. **Schema Design**:
   - `version: String` ("1.0.0").
   - `max_file_bytes: u64` (1 KiB .. 1 GiB).
   - `max_line_bytes: usize` (128 .. 65536).
   - `ignored_dirs: Vec<String>` (1 .. 50 entries).
   - `allow_patterns: Vec<String>` (allowlist regex / string exemptions).
   - `require_clean: bool`.
