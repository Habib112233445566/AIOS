# Task Evidence: T-02027 - Capability Model / CLI surface: Security Review (Phase 2, Sub-Epic 3)

## 1. Overview
- **Task ID**: `T-02027`
- **Phase**: Phase 2 — Security Kernel & PEP Fabric
- **Epic**: Capability Model
- **Sub-Epic**: 3 (CLI surface)
- **Goal**: Perform comprehensive security review and threat modeling of `aiosh capability` CLI surface.

---

## 2. Threat Model Analysis (THREAT-CAPCLI-01..06)

| Threat ID | Threat Description | Attack Vector / Trigger | Severity | Mitigation Strategy |
|---|---|---|---|---|
| `THREAT-CAPCLI-01` | **Subject / Target Injection** | Caller passes control characters or escape sequences in `--subject` or `--scope-target` to poison logs or terminal output. | Medium | Validate that subject and target identifiers contain no control characters or newlines; sanitize terminal output via `sanitize_terminal`. |
| `THREAT-CAPCLI-02` | **Arbitrary File Overwrite via `--store`** | Attacker specifies `--store /etc/shadow` or traversal paths to corrupt system files. | High | Enforce `validate_service_path` rejecting paths $> 1024$ chars, control characters, `..` traversal, and non-JSON extensions. |
| `THREAT-CAPCLI-03` | **Ambient Root Capability Issuance** | Unprivileged agent executes `aiosh capability issue` claiming arbitrary privileges. | Critical | Enforce issuer constraint: `--issuer` must strictly equal `kernel` or start with `admin:*`. |
| `THREAT-CAPCLI-04` | **Terminal Escape Sequence Injection (CWE-150)** | Malicious subject or scope strings contain ANSI/VT100 escape sequences to spoof terminal output. | Medium | Wrap all untrusted text in `sanitize_terminal` before printing to stderr or stdout. |
| `THREAT-CAPCLI-05` | **Audit Evasion or Inconsistent Provenance** | Command execution without emitting an audit row or emitting corrupted classifier fields. | High | Guarantee every code path invokes `classify_and_emit` into `AuditRing` with tool `"capability"`. |
| `THREAT-CAPCLI-06` | **Quota Bypass via Negative or Overflow Numbers** | Passing negative or out-of-range values to `--max-invocations` or `--quota-bytes`. | Low | Use `u64::from_str` parsing; reject negative values or formatting errors with exit code 2. |

---

## 3. Hardening Plan for T-02028
1. Add strict subject identifier validation in `cmd_capability`: reject control characters, newlines, and empty subjects.
2. Ensure integer quota flags (`--max-invocations`, `--quota-bytes`) return exit code 2 on parse errors rather than silently defaulting or crashing.
3. Validate `--scope-target` length and character hygiene in `parse_cli_scope`.
4. Ensure audit rows record detailed error classification for invalid inputs.
