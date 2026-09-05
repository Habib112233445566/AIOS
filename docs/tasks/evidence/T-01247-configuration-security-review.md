# T-01247: Package Management - Configuration: Security Review

## Metadata
- **Task ID:** `T-01247`
- **Subsystem:** `code/aiosh-rust`
- **Component:** Package Management Configuration Security Review
- **Status:** Complete

## 1. Security Architecture & Threat Surface Overview
The Package Management Configuration subsystem controls operational parameters for package storage, entity ceilings, format preferences, and upstream repository network endpoints. It exposes both operator CLI (`aiosh package config`) and autonomous agent MCP (`aios.package.config`) surfaces.

### Core Security Controls:
1. **Bounded File Ingestion (`PC6`)**:
   Configuration files are subject to an upfront metadata size check and a bounded stream reader (`take(65_536 + 1)`), precluding memory exhaustion (OOM) attacks from multi-gigabyte files.
2. **Strict Range & Type Invariants (`PC1..PC4`)**:
   - `store_path`: Enforces non-empty, $\le 1024$ bytes, and rejects ASCII control characters and null bytes (`\0`).
   - `max_store_size_bytes`: Restricted to $[64 \text{ KiB} \dots 100 \text{ MiB}]$.
   - `max_entity_count`: Restricted to $[10 \dots 100,000]$.
   - `allowed_repositories`: Mandatory transport security (`https://` or `file://`). Plaintext `http://` and legacy `ftp://` are rejected.
3. **PEP Capability & Audit Mediation (ADR-0035)**:
   - CLI invocations emit structured audit records via `classify_and_emit` to SQLite `audit.db`.
   - MCP tool calls route through `dispatch::recorded_call`, enforcing Gate #1 (classifier) and Gate #2 (PEP token), writing immutable SHA-256 hash-chained audit rows on all branches.

---

## 2. Abuse Scenarios & Mitigations

### Scenario 1: Path Traversal & Arbitrary File Access via `--config`
- **Attack Vector**: An attacker attempts to read `/etc/shadow` or sensitive files by passing `--config /etc/shadow`.
- **Analysis & Defense**:
  - The configuration loader does not execute arbitrary code or shell subshells.
  - Files are parsed strictly as JSON conforming to `PackageConfig`. Unrelated file content (such as password files, keys, or binary data) fails JSON deserialization, returning `CONFIG_RESOLUTION_FAILED`.
  - Ingestion is size-capped to 64 KiB, preventing buffer bloat.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 2: Network Man-in-the-Middle (MITM) & Package Tampering via Insecure Repositories
- **Attack Vector**: An attacker or rogue configuration sets an unencrypted upstream repository (`http://deb.debian.org/debian` or `http://attacker-mirror.com`) to tamper with package binaries.
- **Analysis & Defense**:
  - Invariant `PC4` actively inspects every URL in `allowed_repositories`.
  - Any URL not prefixed with `https://` or `file://` causes validation failure (`PC4 violation`).
  - Control characters and null bytes in repository URLs are rejected.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 3: Resource Exhaustion via Pathological Store Configurations
- **Attack Vector**: Submitting zero or multi-terabyte sizing limits to crash down-stream algorithms or allocate excessive memory buffers.
- **Analysis & Defense**:
  - `PC2` enforces maximum store size bounds between 64 KiB and 100 MiB.
  - `PC3` restricts entity counts between 10 and 100,000.
  - Zero, negative, or overflow values fail validation immediately before any store initialization.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 4: Audit Trail Circumvention
- **Attack Vector**: An operator or agent crafts an erroneous configuration to execute code without leaving an audit footprint.
- **Analysis & Defense**:
  - Both CLI and MCP paths record audit events on both success and error outcomes.
  - Failed configuration resolution logs the exact error reason, actor ID, and target path to the audit ring.
- **Verdict / Residual Risk**: Fully Mitigated.

---

## 3. Review Conclusion
- Input validation: Comprehensive and verified (`PC1..PC6`).
- Gating and auditing: Verified across CLI and MCP surfaces.
- Policy bypass: No known policy bypasses or unhandled security flaws remain open.
- Status: APPROVED for Hardening (T-01248).
