# T-00759 — Secrets & Access Hygiene / configuration: Documentation

## 1. Operator Documentation
Documented `SecretsConfig` and `docs/secrets_config.json` schema in `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.

### Configuration Example (`docs/secrets_config.json`)
```json
{
  "version": "1.0.0",
  "max_file_bytes": 16777216,
  "max_line_bytes": 4096,
  "ignored_dirs": [
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist"
  ],
  "allow_patterns": [],
  "require_clean": false
}
```

## 2. Invariant Validation
- Ran `python tools/check_task_docs.py` to confirm full compliance with documentation invariants C1..C6.
