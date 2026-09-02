# T-00451 — Documentation Index Control / configuration: Research

## 1. Goal
Establish facts, constraints, schema conventions, and prior art for the configuration layer of Documentation Index Control in `aiosh-core`.

## 2. Facts vs. Assumptions

### Facts (Empirical Repository Context):
1. **Existing Config Models in AIOS**:
   - `toolchain_config.rs` provides `ToolchainManifest` loaded from file path, environment variable (`AIOS_TOOLCHAIN_CONFIG`), or embedded defaults.
   - `ledger_config.rs` provides `LedgerConfig` with strict field bounds and serialization tests.
2. **Documentation Index Scope**:
   - Currently, hardcoded document paths are used across `aiosh-cli` and `aiosh-mcp`.
   - A dedicated `DocIndexConfig` is required to allow operators to declare documentation directories (`root_dirs`), allowed extensions (`include_extensions`), ignored paths (`exclude_patterns`), and strict link validation flags (`enforce_strict_links`).

### Assumptions:
1. When no explicit config file is passed, the system should gracefully load defaults without panicking.
2. Config size should be strictly bounded (e.g. `MAX_CONFIG_BYTES = 64KB`).

## 3. Prior Art & Authoritative Specifications
- **`mkdocs.yml`**: Configures nav paths, docs directory, and validation plugins.
- **`docusaurus.config.js`**: Defines documentation root folders and sidebar rules.
- **AIOS Toolchain & Ledger Config**: Structured JSON schema with path validation and environment overrides.

## 4. Proposed `DocIndexConfig` Schema
```json
{
  "version": "1.0.0",
  "root_dirs": ["docs"],
  "include_extensions": [".md"],
  "exclude_patterns": ["**/node_modules/**", "**/target/**"],
  "enforce_strict_links": true
}
```

## 5. Decisions Needed
1. **Configuration Location**: Default file path `docs/doc_index_config.json` and environment variable `AIOS_DOC_INDEX_CONFIG`.
   - *Decision*: Adopt `docs/doc_index_config.json` with env var fallback.
2. **Validation Rules**: Maximum 50 root directories, non-empty extension lists, capped config size of 64 KiB.
   - *Decision*: Enforce standard AIOS hardening size caps.

## 6. Next Steps
Advance to Specification (T-00452) to define the typed Rust structs, JSON serialization schema, and error envelopes.
