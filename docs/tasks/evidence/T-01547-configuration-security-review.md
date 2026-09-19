# T-01547 — Filesystem Layout configuration: Security Review

## Metadata
- **Task ID:** `T-01547`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target / Filesystem Layout / configuration
- **Status:** Complete — comprehensive security review of configuration validation (D1..D9 / FL1..FL6) and store parsing contracts completed; zero policy bypasses found.
- **Date:** 2026-09-19
- **Depends on:** `T-01546` (Configuration Integration)
- **Feeds:** `T-01548` (Configuration Hardening)
- **Artifacts:** `docs/tasks/evidence/T-01547-configuration-security-review.md`, `docs/tasks/evidence/T-01547-security.md`

---

## 1. Scope & Security Review Objectives

This security review evaluates the configuration surface of the Filesystem Layout subsystem, covering:
1. **Spec Ingestion & Validation (`aiosh_core::fs_layout`)**:
   - Enforcement of D1..D9 decisions and consistency invariants FL1..FL6.
   - Input hygiene (path validation, length bounds, control character rejection).
   - Strict serde deserialization (`deny_unknown_fields`) on spec structures.
2. **Store Ingestion & Persistence (`aiosh_core::fs_layout_service`)**:
   - Schema enforcement on persisted store files (`deny_unknown_fields` on `FilesystemLayoutStore`).
   - Atomic persistence semantics (`.tmp.<pid>`, fsync, atomic rename).
3. **CLI & MCP Gating & Audit Logging (`cmd_fs_layout`, `aios.fs_layout.*`)**:
   - Policy Enforcement Point (PEP) boundary enforcement on all state mutations.
   - Forensic integrity: SHA-256 hash-chained SQLite WAL audit row emission on success, operational error, and policy refusal.

---

## 2. Abuse Scenarios & Verification Matrix

The following abuse scenarios were evaluated across both the CLI (`aiosh layout`) and MCP (`aios.fs_layout.*`) surfaces:

| # | Abuse Scenario | Attack Vector / Malicious Payload | Observed Behavior | Verdict |
|---|---|---|---|---|
| **S1** | **Unknown Field Injection** | Attacker injects rogue top-level or nested JSON fields (e.g. `dry_run: true`, `elevate: true`) into layout spec | Serde rejects document immediately: `invalid layout JSON: unknown field '...'`. No partial parse; fail-closed. | **PASS** (Protected by D1 / C9) |
| **S2** | **Permission Mask Neutralization** | Attacker specifies directory `mode: 0` (unusable/denial-of-service) or `mode: 0o10000` (overflow/unit confusion) | Validator rejects with `directory '...' mode must be in 1..=0o7777 (octal), found ...`. | **PASS** (Protected by D2 / E-2) |
| **S3** | **CIS Security Option Stripping** | Attacker omits `nodev`, `nosuid`, or `noexec` from `/tmp` or `/dev/shm` mounts | Validator rejects with `FL4 violation: mount '...' missing mandatory security option '...'`. | **PASS** (Protected by D3 / E-3) |
| **S4** | **UsrMerge Path Traversal / Symlink Hijack** | Attacker specifies `symlink_target: "/etc/shadow"` (absolute) or `"usr/../../etc/shadow"` (escaping traversal) | Validator rejects: `symlink_target '...' must be a relative path under 'usr' (UsrMerge)`. Traversal and absolute targets blocked. | **PASS** (Protected by D4 / E-4) |
| **S5** | **Timestamp Normalization Bypass** | Attacker supplies non-UTC timestamp (e.g. `+02:00` offset) to cause desynchronization or key mismatch | Validator requires RFC 3339 UTC ending in `Z`. Non-UTC forms rejected. | **PASS** (Protected by D7 / E-5) |
| **S6** | **fstab Dump Field Overflow** | Attacker supplies arbitrary integer in `dump` field (e.g. `dump: 99`) | Validator enforces fstab(5) field 5 semantics: `dump` must be `0` or `1`. | **PASS** (Protected by D7 / E-6) |
| **S7** | **Zombie / Dead-Mount Spec** | Attacker specifies layout where no mount is marked `required == true` | Validator rejects: `FL6 violation: at least one mount must be marked required`. | **PASS** (Protected by D8 / E-7) |
| **S8** | **Store Tampering via Unknown Keys** | Attacker adds rogue keys (e.g. `active_layout`) to layout store JSON file | Store deserialization fails closed with `failed to deserialize layout store from ... unknown field '...'`. Mutations refused without modifying store. | **PASS** (Protected by T-01544 / C9) |
| **S9** | **PEP Authorization Bypass** | Caller without explicit PEP grant invokes mutating tools (`register`, `set_active`, `remove`, `import_fstab`) | PEP gate halts execution before reading payload; returns `outcome=refused` with honest audit row. | **PASS** (Protected by PEP / ADR-0035) |
| **S10** | **Path Traversal in Store Argument** | Caller passes `store_path` escaping authorized directories (e.g. `../../sensitive.json`) | Enforced by PEP `scope.paths` matching canonical real paths. Unscoped access denied. | **PASS** (Protected by T-01537 F-1 fix) |
| **S11** | **Denial of Service via Oversized Spec** | Attacker submits massive payload (> 10 MiB) or infinite FIFO stream | Capped at 10 MiB during read; non-regular files (FIFOs, char devices) refused upfront. | **PASS** (Protected by FL7 / MAX_LAYOUT_DOC_BYTES) |
| **S12** | **Audit Trail Integrity & Fail-Closed Behavior** | Attacker forces deliberate validation or parse failures | Every failure branch writes an honest audit record to SQLite WAL with exact error message and caller context. | **PASS** (Protected by FL4 / C2) |

---

## 3. Findings Summary

- **Policy Bypasses Found:** 0 (Zero).
- **Vulnerabilities Open:** None.
- **Security Posture:** The configuration validation layer (D1..D9, FL1..FL6) and store parsing logic strictly enforce fail-closed semantics across both operator CLI and agent MCP surfaces.
- **Recommendations for Hardening (T-01548):**
  1. Maintain explicit size caps across all configuration deserialization entry points.
  2. Continue strict verification that temporary staging files (`.tmp.<pid>`) are cleaned up even when write/rename operations fail.
  3. Ensure that all error envelopes preserve sanitized error strings without leaking internal memory layouts or system environment variables.

---

## 4. Acceptance Confirmation

- [x] Security evidence file exists with abuse scenarios.
- [x] No known policy bypass remains open.
