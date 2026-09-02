# T-00377 — Dependency & Toolchain Pinning / security policy: Security Review

## 1. Review Overview
This security review assesses the security policy enforcement, threat model, input validation, and abuse scenarios governing Dependency & Toolchain Pinning in AIOS.

## 2. Security Controls & PEP Architecture
1. **PEP Fail-Closed Mechanism**:
   - Mutating toolchain actions (`aios.toolchain.set`, `toolchain.set`) are classified as `is_irreversible` in `code/aiosh-rust/aiosh-core/src/pep.rs`.
   - `check_toolchain_policy` verifies grant tokens, ensuring that autonomous agents cannot alter build or runtime version constraints without explicit user-delegated cryptographic grants.
2. **Audit Logging Invariants**:
   - All toolchain operations write immutable records to the SQLite WAL audit ring. Refusals, errors, and successes are all recorded honestly per ADR-0035 §F-2.
3. **Input Validation & Size Caps**:
   - Configuration files are bounded by a 64KB maximum file size cap, mitigating denial-of-service memory exhaustion vectors.
   - Vectorized process execution (`Command::new("rustc").arg("-V")`) prevents shell interpolation attacks.

## 3. Abuse Scenarios & Mitigations

### Abuse Scenario 1: Unauthorized Toolchain Version Downgrade
- **Attack Vector**: An agent attempts to downgrade compiler versions to inject vulnerable or unpatched runtime dependencies.
- **Mitigation**: The PEP dispatcher and `check_toolchain_policy` intercept the mutation request, verify the absence of an active grant token, and refuse execution with an explicit error. A refused audit row is logged.

### Abuse Scenario 2: Denial of Service via Huge Manifest Payload
- **Attack Vector**: Hostile process writes an oversized multi-megabyte JSON file as the toolchain config.
- **Mitigation**: The loader reads at most 64KB and aborts parsing immediately if the file size exceeds bounds.

### Abuse Scenario 3: Binary Version Spoofing / Malicious Environment
- **Attack Vector**: A subverted environment alters `PATH` to point to a malicious `rustc` binary that emits arbitrary text.
- **Mitigation**: The toolchain enforcement service parses exact version outputs, enforces bounded 15-second execution timeouts with child termination, and strictly compares version tokens against the pinned manifest.

## 4. Conclusion
No known security policy bypasses remain open. The toolchain security policy maintains strict fail-closed enforcement and tamper-proof audit trails.
