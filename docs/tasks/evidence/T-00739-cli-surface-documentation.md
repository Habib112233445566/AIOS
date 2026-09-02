# T-00739 — Secrets & Access Hygiene / CLI surface: Documentation

## 1. Operator Documentation & Invocation
Documented `aiosh secrets <scan|check>` in `docs/README.md` under `## Secrets & Access Hygiene (T-00711..T-00810)`.

### Example Commands
```bash
# Scan single file
aiosh secrets scan --file code/aiosh-rust/Cargo.toml

# Fast check mode (exit 0 on clean, 1 on finding)
aiosh secrets check

# Machine-readable JSON output
aiosh secrets scan --json
```

## 2. Invariant Validation
- Ran `python tools/check_task_docs.py` to ensure compliance across all doc invariants C1..C6.
