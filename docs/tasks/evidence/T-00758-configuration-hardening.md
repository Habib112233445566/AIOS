# T-00758 — Secrets & Access Hygiene / configuration: Hardening

## 1. Hardening Deliverables
- **Bounded Stream Ingestion**: `SecretsConfig::from_path` reads configuration files via `take(MAX_CONFIG_BYTES)` ensuring no oversized input can cause memory spikes.
- **Defensive Struct Constraints**:
  - `version`: Non-empty, $\le 32$ chars.
  - `max_file_bytes`: $1024 \le n \le 1073741824$.
  - `max_line_bytes`: $128 \le n \le 65536$.
  - `ignored_dirs`: $1 \le \text{count} \le 50$.
  - `allow_patterns`: $\le 100$.
- **Explicit Error Envelopes**: Parsing or validation errors return auditable messages in CLI and MCP standard envelopes.
