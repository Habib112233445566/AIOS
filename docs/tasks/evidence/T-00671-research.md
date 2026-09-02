# T-00671 — Repository Health / security policy: Research

## Facts
- `check_security_governance` fully implemented in `repo_health_service.rs:137-178`.
- Checks: SECURITY.md existence, ≥100 chars, no TODO markers.
- Already integrated into CLI and MCP surfaces via `check_repo_health` orchestrator.
- No new code needed; sub-epic validates existing coverage.
