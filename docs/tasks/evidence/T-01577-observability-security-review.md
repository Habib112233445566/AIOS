# T-01577 — Filesystem Layout observability: Security Review

## Metadata
- **Task ID:** `T-01577`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / observability
- **Status:** In Progress — security review initiated for the Filesystem Layout observability subsystem.
- **Date:** 2026-09-19
- **Depends on:** `T-01576` (Integration)
- **Feeds:** `T-01578` (Observability Hardening)
- **Artifacts:** `docs/tasks/evidence/T-01577-observability-security-review.md`, `docs/tasks/evidence/T-01577-security.md`

---

## 1. Security Review Scope & Objectives

Review the security properties of the Filesystem Layout observability and telemetry interfaces:
1. Input validation, escape injection, and log poisoning (CWE-150).
2. Telemetry completeness and non-repudiation (ADR-0035 hash chaining).
3. Information disclosure risks in audit records and status envelopes (CWE-209).
4. Resource consumption and telemetry flood resilience (CWE-400).

---

## 2. Abuse Scenarios Analyzed

### Scenario O-A1: Telemetry Suppression / Audit Dropping
- **Vector**: Attacker triggers state transitions while attempting to bypass audit logging to hide unauthorized layout changes.
- **Defense**: Audit emission is built directly into the dispatch pipeline for both CLI (`cmd_fs_layout`) and MCP (`call_mcp_tool`). Both success and refusal paths emit an audit record before completing the response.
- **Verdict**: Mitigated — tested by FL12 O1.

### Scenario O-A2: Audit Poisoning via Control Characters & Format Strings (CWE-150)
- **Vector**: Caller supplies crafted strings (e.g. `\n`, `\r`, `\x00`, ANSI escapes, or JSON-like syntax) in layout IDs, partition names, or error messages to forge fake log entries.
- **Defense**: SQLite WAL audit ring records entries as typed columns, and CLI/MCP JSON outputs serialize all string parameters using JSON-escaped UTF-8.
- **Verdict**: Mitigated — tested by FL4 and FL12.

### Scenario O-A3: Sensitive Data Exfiltration via Observability Streams (CWE-209)
- **Vector**: Observability endpoints leak secret configuration or encryption credentials via audit logs or status query responses.
- **Defense**: Filesystem layout profiles specify structural partition dimensions, mount options, and directory modes. No user credentials, secret keys, or raw partition data are logged.
- **Verdict**: Mitigated — verified by review.

### Scenario O-A4: Observability Denial of Service (CWE-400)
- **Vector**: Repeated queries flood SQLite WAL audit ring or exhaust memory during audit tail retrieval.
- **Defense**: Audit queries enforce bounded query limits (`-n 60` default), and write transactions use bounded WAL locks.
- **Verdict**: Mitigated — tested by FL8 and FL12.

---

## 3. Initial Assessment
- All 4 abuse scenarios O-A1 through O-A4 are addressed by current safeguards and tested in FL4, FL8, and FL12.
- No open policy bypasses or unhandled security defects identified.
