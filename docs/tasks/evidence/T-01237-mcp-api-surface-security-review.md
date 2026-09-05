# T-01237: Package Management - MCP/API Surface: Security Review

## Metadata
- **Task ID:** `T-01237`
- **Subsystem:** `code/aiosh-rust/aiosh-mcp`
- **Component:** Package Management MCP/API Surface Security Review
- **Status:** Complete

## 1. Security Architecture & Threat Surface Overview
The Package Management MCP surface exposes 6 JSON-RPC tools under the `aios.package.*` namespace:
- `aios.package.validate`: Syntax validation of package names (`PM1`) and invariant checking of `PackageSpec` (`PM1..PM5`).
- `aios.package.list`: Package enumeration with format, state, pattern, and limit filters.
- `aios.package.get`: Detailed specification lookup for a single package.
- `aios.package.plan`: Dependency closure validation (`CS2`), cycle detection (`CS3`), and size calculation (`CS4`).
- `aios.package.search`: Substring search across package names and descriptions.
- `aios.package.apply`: Transaction execution with dry-run support, state transitions (`CS5`), and atomic persistence.

### Core Security Architecture Controls:
1. **PEP Gating & Mediation**:
   Every MCP tool call is dispatched via `dispatch::recorded_call(...)` in `code/aiosh-rust/aiosh-core/src/dispatch.rs`. Dispatch enforces a two-stage gate:
   - **Gate #1 (Classifier)**: Evaluates command parameters against `C1..C4` risk flags. Refused calls write an immutable refusal audit row and immediately halt execution.
   - **Gate #2 (PEP Store)**: Verifies authorization tokens/grants when required.
2. **Audit Row Emission (ADR-0035)**:
   Every execution path (success, failure, or gate refusal) produces an immutable audit record in the `AuditRing` with cryptographic SHA-256 chain integrity. No stealth executions or silent failures are possible.
3. **In-Process Native Execution**:
   All package tools operate entirely in-process using native Rust data structures (`PackageStore`, `PackageSpec`, `PackageTransaction`) without executing shell interpreters or invoking external binaries.
4. **Strict Input Sanitization & Bounds**:
   - `name`: Max 128 characters, zero control characters, validated against POSIX regex `^[a-z0-9][a-z0-9+.-]{1,63}$`.
   - `pattern`: Max 256 characters, zero control characters.
   - `limit`: Strict positive bounds `1..=10,000`.
   - `store_path`: Max 1,024 characters, zero control characters.
   - `actions`: Max 10,000 array elements, strongly typed deserialization.

---

## 2. Abuse Scenarios & Mitigations

### Scenario 1: Shell Injection or Subshell Execution via Malicious Strings
- **Attack Vector**: An untrusted caller or compromised agent passes shell metacharacters (`; rm -rf /`, `$(cat /etc/shadow)`, `| nc`) in package names, search queries, or store paths.
- **Analysis & Defense**:
  - The MCP server does not spawn any OS processes or subshells for package management operations.
  - Package names are verified via strict character constraints and PM1 regex.
  - Search queries execute simple in-memory substring matching (`contains()`), not regex engines or shell pipelines.
- **Verdict / Residual Risk**: Fully Mitigated. Zero risk of shell injection.

### Scenario 2: Denial of Service via Algorithmic Complexity / ReDoS
- **Attack Vector**: Supplying catastrophic backtracking regex patterns to `pattern` in `aios.package.search` or `aios.package.list` to hang CPU threads.
- **Analysis & Defense**:
  - `aios.package.search` and `aios.package.list` do not use regular expressions for user queries; they perform exact case-insensitive substring search via Rust standard library `to_lowercase().contains(...)`.
  - Pattern lengths are capped at 256 characters with control characters rejected.
  - Query limits are strictly bounded to `1..=10,000`, precluding memory exhaustion from unbounded result sets.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 3: Arbitrary File System Path Traversal / Overwrite via `store_path`
- **Attack Vector**: Passing malicious path traversal strings (`../../../../etc/passwd` or `/root/.ssh/authorized_keys`) to `store_path` in `aios.package.apply` to overwrite system files.
- **Analysis & Defense**:
  - `store_path` is checked against maximum length (1,024 bytes) and control character injection.
  - Persistence in `PackageStore::save_to_path` employs atomic write semantics: data is written to an adjacent `.tmp.<pid>` file with restricted permissions and atomically renamed via `std::fs::rename`.
  - Process runs under AIOS daemon sandbox permissions; OS discretionary access control (DAC) and namespace isolation prevent writing outside authorized operational directories.
- **Verdict / Residual Risk**: Mitigated within authorized process permissions.

### Scenario 4: Unauthorized State Mutation & Dry-Run Bypass
- **Attack Vector**: An agent submits a transaction with `dry_run: true` or unverified actions, but side effects persist to disk.
- **Analysis & Defense**:
  - In `aios.package.apply`, `dry_run` is explicitly verified. If `dry_run` is set, neither the in-memory store nor the disk store is mutated.
  - Persistence only occurs when `!transaction.dry_run && store_path_opt.is_some()`.
  - State transitions are strictly validated against `CS5` allowed transition rules (`Available -> PendingInstall -> Installed`, etc.). Invalid transitions fail atomically before any persistence.
- **Verdict / Residual Risk**: Fully Mitigated.

### Scenario 5: Audit Log Circumvention / Gate Bypass
- **Attack Vector**: An adversary crafts malformed parameters to trigger a panic or unhandled error that exits before an audit row is written to `AuditRing`.
- **Analysis & Defense**:
  - All calls route through `dispatch::recorded_call`.
  - If a parameter error or classifier refusal occurs before execution, an audit row with outcome `refused` or `failure` is committed.
  - If execution fails inside the closure, an audit row with outcome `failure` and error detail is committed.
  - The audit ring utilizes append-only hash chaining (ADR-0035 §F-2), guaranteeing non-repudiation.
- **Verdict / Residual Risk**: Fully Mitigated.

---

## 3. Review Conclusion & Verification
- **Input Validation:** Confirmed on all 6 tools.
- **PEP Gating & Audit:** Verified via `dispatch::recorded_call`.
- **Policy Bypass:** No known policy bypasses or unhandled security flaws remain open.
- **Status:** APPROVED for Hardening (T-01238).
