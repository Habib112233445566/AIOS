# T-00129 — CI Smoke Orchestration / core service: Documentation

**Date:** 2026-08-25
**Feature:** CI Smoke Orchestration core service

## 1. Documentation Updates
- Updated `docs/README.md` to include a new subsection `### CI Summary Service (T-00121..T-00130)` under the CI Smoke Orchestration section.
- Explained what shipped: the core service was implemented natively in Rust (`aiosh-core`) and exposed via the `aiosh ci` CLI command, aligning with the v2.1 shipping stack mandate.

## 2. Example Tool Calls
Copy-pasteable examples were added for operators and agents to invoke the service directly:
```bash
# Validate the CI output artifact explicitly (defaults to /tmp/aiosh-ci-results.json)
aiosh ci check

# Display a human-readable run report
aiosh ci show

# List only the failing suites and their log paths
aiosh ci failures
```

## 3. Honest Limitations
Documented the following limitations explicitly in the README:
- Bounded retries handle orchestrator lock contention but assume the artifact will eventually exist.
- The artifact JSON payload is read completely into memory (capped at 1MB to prevent OOM) rather than streamed.

## 4. Evidence Links
Linked the relevant evidence paths directly in the README documentation section to ensure a traceable chain of custody for the feature's development (e.g., `tasks/evidence/T-00126-core-service-integration.md`).
