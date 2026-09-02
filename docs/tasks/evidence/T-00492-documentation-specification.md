# T-00492 — Documentation Index Control / documentation: Specification

## 1. Specification Overview
This specification defines the documentation content, structure, interface references, and copy-pasteable examples for Documentation Index Control in AIOS.

## 2. Documentation Structure & Specification

### A. CLI Commands Reference (`aiosh-cli`)
- **`aiosh doc show [--json] [--config <path>]`**:
  - Displays the indexed documentation manifest (version, sections, titles, file paths).
- **`aiosh doc check [--json] [--config <path>]`**:
  - Validates all markdown link targets across indexed documentation files and returns link integrity reports with aggregate telemetry.
  - Returns exit code 0 if all links are valid; exit code 1 if broken links or path traversal escapes are detected.
- **`aiosh doc search <query> [--json] [--config <path>]`**:
  - Searches indexed documents by case-insensitive title, section, or relative path substring.

### B. MCP Tools Reference (`aiosh-mcp`)
- **`aios.doc.index.get`**:
  - Arguments: `repo_path` (string, default: `"."`), `config_path` (string, optional).
  - Returns: `DocIndexManifest` JSON payload.
- **`aios.doc.check`**:
  - Arguments: `repo_path` (string, default: `"."`), `config_path` (string, optional).
  - Returns: `report` (`DocLinkValidationReport`) and `telemetry` (`DocIndexTelemetry`).
- **`aios.doc.search`**:
  - Arguments: `query` (string, required), `repo_path` (string, default: `"."`), `config_path` (string, optional).
  - Returns: `matches` (array of `DocIndexEntry`), `count` (integer).

### C. Configuration Schema (`DocIndexConfig`)
```json
{
  "root_dirs": ["docs"],
  "include_extensions": [".md", ".markdown"],
  "exclude_patterns": ["node_modules", "target", ".git"],
  "enforce_strict_links": true
}
```

### D. Security & Hardening Invariants
- File ingestion ceiling: 16 MiB per document.
- Configuration file ceiling: 64 KiB.
- Repository root containment: Out-of-tree traversal links (`..`) fail validation and are logged as security errors.
- PEP token gating: Mutating operations (`aios.doc.set`, `doc.set`) require active verified PEP grant tokens.
