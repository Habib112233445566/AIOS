# Task Evidence: T-02227 (Grant Lifecycle / CLI surface: Security Review)

## 1. Metadata
- **Task ID:** `T-02227`
- **Subsystem:** Phase 2 — Security Kernel & PEP Fabric
- **Component:** Grant Lifecycle CLI Surface Security Review (`code/aiosh-rust/aiosh-cli`)
- **Status:** Complete
- **Date:** 2026-09-23
- **Milestone:** Sub-Epic: Grant Lifecycle (3/10) — CLI Surface Security Review

---

## 2. Threat Modeling & Attack Surface Analysis

The `aiosh pep grant` CLI surface provides command-line control over zero-ambient authorization grants, delegation, validation, and lifecycle transitions. A comprehensive threat analysis evaluated the following vectors:

### 2.1 Abuse Scenarios & Mitigations

| Scenario ID | Attack Vector | Potential Impact | Implemented Mitigation | Verification Status |
|:---|:---|:---|:---|:---|
| **AS-GRANT-01** | Path Traversal / Arbitrary File Overwrite via `--store` | Arbitrary file read/write across OS directories using `../` or absolute symlinks | Strict path hygiene enforced by `validate_pep_service_path`: requires `.json` extension, strictly forbids `..`, and constrains path length ($\le 1,024$) | PASS (Exit code 2, error enveloped) |
| **AS-GRANT-02** | Authority Escalation via Right Expansion in Attenuation | Child grant attempts to acquire rights outside parent grant's authorized scope (e.g. parent has `Read`, child requests `Write`) | `PepGrantService::attenuate_grant` enforces strict rights subset containment; rejects non-subset delegations | PASS (Exit code 1, `ATTENUATION_FAILED`) |
| **AS-GRANT-03** | Delegation Loop & Unbounded Delegation Chain | Child grant attempts infinite re-delegation beyond designated depth | `max_delegation_depth` is strictly checked ($> 0$), required `delegate` right verified, and depth decremented by 1 at each derivation step | PASS (Exit code 1 on depth exhaustion) |
| **AS-GRANT-04** | Use-After-Revocation / Dangling Child Authority | Attenuated child grant remains active and usable after parent or ancestor revocation | `PepGrantService::revoke_grant` cascades revocation recursively to all descendant grants; `validate` verifies state is `Active` and checks ancestor chain | PASS (Exit code 1 on revoked grant) |
| **AS-GRANT-05** | Temporal Window Bypass | Grant used before `not_before` or after `expires_at` | `validate_grant_usage` and CLI `validate` with `--now` enforce rigorous UTC bounds comparisons | PASS (Exit code 1 on expired grant) |
| **AS-GRANT-06** | Terminal Control Character / ANSI Escape Injection | Malicious error strings or identifiers containing ANSI escapes manipulate operator console | All stdout/stderr terminal formatting filters untrusted data via `sanitize_terminal` | PASS (Sanitization enforced) |
| **AS-GRANT-07** | Audit Suppression / Silent Mutation | Mutating operations (issue, attenuate, revoke, sweep) execute without cryptographic ledger trace | Every path dispatches through `classify_and_emit` / `dispatch::recorded_call`, committing an immutable SHA-256 chained row into SQLite `$AIOSH_HOME/audit.db` | PASS (Audit row confirmed for all paths) |

---

## 3. Policy Bypass Verification
- Code analysis confirmed zero unauthenticated grant derivation paths.
- No path exists where an expired, revoked, or non-subset grant evaluates to `valid: true`.
- Zero policy bypass vulnerabilities identified.

---

## 4. Acceptance Confirmation
- [x] Security review document authored covering all CLI grant lifecycle subcommands.
- [x] 7 abuse scenarios analyzed, mitigated, and verified.
- [x] No open policy bypass remains.
- [x] Audit row emission verified for every state-changing and query path.
