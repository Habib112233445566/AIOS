# T-00679 — Repository Health / security policy: Documentation

## Documented Features
- `check_security_governance(repo_root)` validates root `SECURITY.md`.
- Checks: existence, ≥100 chars, no `TODO` markers.
- Integrated into `check_repo_health` orchestrator, accessible via `aiosh repo health` CLI and `aios.repo.health` MCP tool.
- Independent validation via `python tools/check_security_policy.py` (S1..S5).
