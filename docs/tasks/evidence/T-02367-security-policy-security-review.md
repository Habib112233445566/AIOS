# Task Evidence: T-02367 (Audit Chain Extensions / security policy: Security Review)

## 1. Metadata
- **Task ID:** `T-02367`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Audit Chain Extensions Security Policy Security Review
- **Status:** Complete
- **Date:** 2026-09-28
- **Milestone:** Sub-Epic 7: Security Policy (7/10) — Security Review

---

## 2. Threat Analysis & Abuse Scenarios

### 2.1 Scenario 1: Unsigned Administrative / Kernel Mutation Injection
- **Attack Vector**: An unprivileged agent attempts to record a high-impact operation (e.g., `kernel:reboot`, `sec:drop_tables`) without possessing a cryptographic signing key.
- **Analysis**: In `AuditChainSecurityPolicy::evaluate_event`, `signature_required_prefixes` inspects tool names against protected namespaces (`["kernel:", "sec:", "admin:", "pep:"]`). If a signature or public key is missing or blank, evaluation returns `AUDITPOL_ERR_SIGNATURE_REQUIRED`.
- **Verdict**: Fail-closed denial in `Enforcing` mode; prevented.

### 2.2 Scenario 2: Oversized Policy Payload / Heap Exhaustion (DoS)
- **Attack Vector**: An attacker provides a 500 MiB malicious JSON policy file to induce Out-Of-Memory (OOM) crashes in the agent or kernel daemon.
- **Analysis**: `AuditChainSecurityPolicy::load_from_file` inspects filesystem metadata *before* reading contents: `metadata.len() > MAX_AUDIT_SECURITY_POLICY_BYTES` (64 KiB) rejects the file with `AUDITPOL_ERR_VALIDATION`.
- **Verdict**: Mitigated before parsing.

### 2.3 Scenario 3: Anonymous Provenance Spoofing
- **Attack Vector**: An attacker submits audit events with `actor: ""` or `actor: "anonymous"` to obscure attribution during forensic investigations.
- **Analysis**: `disallow_anonymous: true` inspects `actor` and `tool` after trimming whitespace. Empty strings and case-insensitive `"anonymous"` yield `AUDITPOL_ERR_DENIED`.
- **Verdict**: Fail-closed denial; prevented.

### 2.4 Scenario 4: Future Clock Skew Spoofing
- **Attack Vector**: An attacker logs events dated far in the future (e.g., year 2099) to distort log retention indexing and timeline reconstruction.
- **Analysis**: `base.ts` is parsed as RFC 3339; if `ts_epoch > current_epoch + allow_future_timestamps_max_secs` (300s), evaluation fails with `AUDITPOL_ERR_TEMPORAL`.
- **Verdict**: Clock-skew spoofing bounded to $\le 300\text{s}$.

### 2.5 Scenario 5: Causal DAG Inflation & Malformed Hash Injection
- **Attack Vector**: An attacker injects thousands of causal parent links with non-hex or SQL injection strings in `parent_event_hash`.
- **Analysis**: `evaluate_causal_links` enforces link count $\le \text{max\_allowed\_causal\_links}$ (default 32) and checks that each `parent_event_hash` is exactly 64 ASCII hex characters.
- **Verdict**: Graph bounded; SQL/string injection thwarted.

---

## 3. PEP Gating & Audit Invariants
- `aiosh audit policy` is read-only inspection.
- In `aiosh-mcp`, `aios.audit.policy` runs inside `dispatch::recorded_call`, writing exactly one canonical audit row.
- `AuditChainService::record_event` evaluates policy prior to database writes.

---

## 4. Acceptance Confirmation
- [x] Input validation, bounds limits, and untrusted-content handling evaluated.
- [x] Abuse scenarios documented and mitigated.
- [x] No known security policy bypass remains open.
