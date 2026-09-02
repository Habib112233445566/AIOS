# T-01017 — Distro Selection & Justification / Core Service: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Path Traversal / Store Path Hijacking
- **Threat**: Attackers supply arbitrary or malicious `--store` paths containing relative traversal (`../../etc/passwd`) or symlinks to overwrite system files or load untrusted registry profiles.
- **Mitigation & Verification**: `DistroStore::save_to_path` uses atomic tempfile replacement (`.with_extension("tmp")`), and filesystem operations respect OS and user sandbox permission boundaries.

### AS-2: Untrusted Distribution Profile Registration Bypass
- **Threat**: Malicious actor attempts to register distro profile with malicious shell commands in package lists or illegal characters in ID to poison build scripts.
- **Mitigation & Verification**: All profile registrations in `DistroStore::register_profile` mandatorily pass through `validate_distro_profile(&profile)` before insertion, preventing illegal identifiers and malformed version specifications.

### AS-3: Audit Trail Tampering & Non-Repudiation Bypass
- **Threat**: Calling CLI commands or MCP tools without emitting an audit row into the append-only SQLite WAL ledger.
- **Mitigation & Verification**: Every CLI subcommand route in `cmd_distro` calls `classify_and_emit` to log invocations with operator actors, timestamps, and parameters. Every MCP dispatch runs through `dispatch::recorded_call` ensuring strict PEP token verification and hash-chained audit logging.

### AS-4: Evaluation Score Manipulation / Fake Production Readiness
- **Threat**: Malicious actor manipulates distribution evaluation scoring weights to mark an unvetted or insecure distribution as `is_production_ready: true`.
- **Mitigation & Verification**: `DistroEvaluation::evaluate` computes deterministic weighted scores directly from immutable profile properties (binary compatibility 40%, security 30%, footprint 30%), requiring `overall_score >= 0.75` and `binary_compatibility_score >= 0.8` for production authorization.

## 2. Policy Compliance & Invariant Check
- PEP gating: Active on all MCP surfaces.
- Audit row emission: Verified across all CLI subcommands and MCP endpoints.
- No open policy bypass remains.
