# T-00587 — Evidence & Audit Trail / observability: Security Review

## 1. Security Review Scope
This task conducts a threat model and abuse analysis on Evidence & Audit Trail observability interfaces, focusing on log payload clamping, denial-of-service protections, and telemetry integrity.

## 2. Threat Model & Abuse Scenarios

### Scenario O-1: Denial of Service via Telemetry Payload Expansion
- **Threat**: An attacker triggers massive numbers of checksum failures or missing files to flood audit WAL entries and consume disk or memory.
- **Finding & Mitigation**:
  - Audit logging applies `clamp_str(512)` to outcome descriptions, truncating large lists safely with an ellipsis.
  - Manifest validation bounds `records.len() <= 10000` to prevent memory exhaustion during serialization.

### Scenario O-2: Metric Forgery & Silent Corruption
- **Threat**: Forging `EvidenceTelemetry` to report `is_healthy = true` when disk artifacts are missing or corrupted.
- **Finding & Mitigation**:
  - `collect_evidence_telemetry()` computes `is_healthy` directly from `report.is_valid` (`missing_files.is_empty() && hash_mismatches.is_empty()`).
  - Hash comparisons execute constant-time lowercase SHA-256 verification.

### Scenario O-3: Information Disclosure in Diagnostic Logs
- **Threat**: Diagnostic verification reports inadvertently log environment variables, absolute machine usernames, or sensitive repository secrets.
- **Finding & Mitigation**:
  - All paths are strictly validated as repository-relative (`build_evidence_record`).
  - Diagnostics report only relative file paths, expected vs. actual digests, and standard OS read error strings.

## 3. Verdict
- **Status**: PASS
- **Open Vulnerabilities**: 0
- **Residual Risks**: None identified.
