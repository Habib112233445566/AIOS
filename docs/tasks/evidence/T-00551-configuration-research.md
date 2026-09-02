# T-00551 — Evidence & Audit Trail / configuration: Research

## 1. Goal
Establish facts, constraints, and prior art for the configuration layer of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Configuration Precedence Hierarchy**: Existing configuration systems (`doc_index_config`, `toolchain_config`, `ledger_config`) follow strict layered precedence:
   - Environment variables (highest).
   - Explicit file path via argument / env (`AIOS_EVIDENCE_CONFIG_PATH`).
   - Standard repository config file (`config/evidence.config.json`).
   - In-memory fail-safe defaults (lowest).
2. **Defensive Bounds**:
   - `max_file_bytes`: 16 MiB (16,777,216 bytes).
   - `max_config_bytes`: 64 KiB (65,536 bytes) to prevent config poisoning.
   - `evidence_dir`: `"docs/tasks/evidence"`.
   - `allowed_extensions`: `[".md", ".json"]`.
   - `enforce_checksum`: boolean (default: `true`).

### Assumptions:
1. `EvidenceConfig` will live in `code/aiosh-rust/aiosh-core/src/evidence_config.rs`.
2. Serializes/deserializes with `serde` to canonical JSON and provides `from_path`, `from_env`, and `validate` methods.

## 3. Prior Art & Authoritative Sources
- **`doc_index_config.rs` & `ledger_config.rs`**: Canonical configuration loading pattern in `aiosh-core`.
- **12-Factor App (Config)**: Environment-variable driven runtime overrides.

## 4. Decisions Needed
1. Structure of `EvidenceConfig`:
   - `max_file_bytes: u64`
   - `evidence_dir: String`
   - `allowed_extensions: Vec<String>`
   - `enforce_checksum: bool`
   - `require_all_steps: bool`
2. Module location: `code/aiosh-rust/aiosh-core/src/evidence_config.rs`.

## 5. Next Steps
Advance to Specification (T-00552) to formalize the schema and error behaviors.
