# T-00757 — Secrets & Access Hygiene / configuration: Security Review

## 1. Threat Model & Abuse Analysis

| Scenario ID | Attack Vector | Mitigation / Verification | Status |
|---|---|---|---|
| **CFG-1** | Memory Exhaustion via Gigabyte Config File | Config file loading in `SecretsConfig::from_path` reads at most `MAX_CONFIG_BYTES = 64 KiB` before deserialization. | Mitigated |
| **CFG-2** | Unlimited Ignored Directories DoS | Validation rejects configs with $>50$ ignored directory entries or blank directory names. | Mitigated |
| **CFG-3** | Unbounded Max File Size Overflow | `validate()` restricts `max_file_bytes` to range $[1024, 1073741824]$ bytes ($1 \text{ KiB} .. 1 \text{ GiB}$). | Mitigated |
| **CFG-4** | Malformed JSON Injection | Deserialization uses `serde_json` with typed deserialization followed by mandatory semantic validation. | Mitigated |

## 2. Policy Invariants
- **Read-Only Inspection**: Configuration files are opened in read-only mode without mutating the host filesystem.
- **Fail-Closed Strategy**: Any syntax error or validation violation in `SecretsConfig` aborts CLI/MCP execution immediately with an explicit error.
