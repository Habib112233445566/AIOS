# T-00691 — Repository Health / documentation: Research

## Facts (Verified from Source Code & Docs)
1. **Existing Documentation**:
   - `docs/README.md` contains a dedicated `## Repository Health (T-00611..T-00710)` section documenting data model types (`RepoHealthReport`, `RepoHealthCheck`, `HealthStatus`, `HealthCategory`), service checks, configuration options, and automated test runners.
   - Documentation structural integrity is mechanically enforced by `tools/check_task_docs.py` (criteria C1..C6).
2. **Surface Coverage**:
   - CLI subcommands (`aiosh repo health`, `aiosh repo check`) and MCP JSON-RPC tools (`aios.repo.health`, `aios.repo.check`) have documented schemas and execution commands.
   - Formatter functions produce both tabular human-readable prose summaries and raw structured JSON.
3. **Traceability**:
   - Evidence files span `docs/tasks/evidence/T-00611-*.md` through `T-00690-*.md`, preserving an unbroken audit log.

## Assumptions
- Operator documentation in `docs/README.md` should maintain frozen section anchor boundaries and avoid volatile snapshot counts per doc invariant C6.
- Code docstrings and Rust module-level comments in `code/aiosh-rust/aiosh-core/src/repo_health.rs` and `repo_health_service.rs` serve as the internal API reference contract.

## Unknowns & Decisions Needed
- Decision: Add helper formatting functions (`format_repo_health_summary`) to format human-readable CLI outputs if not already present, or verify existing formatting.
