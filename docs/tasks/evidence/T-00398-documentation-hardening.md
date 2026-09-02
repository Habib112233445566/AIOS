# T-00398 — Dependency & Toolchain Pinning / documentation: Hardening

## 1. Hardening Scope
This task hardens the Dependency & Toolchain Pinning documentation by explicitly presenting all resource boundaries, timeout envelopes, truncation caps, and fail-closed behaviors to prevent operator or agent misconfigurations.

## 2. Hardening Measures in Documentation
1. **Explicit Operational Constraints**:
   - Documents the **15-second subprocess execution timeout** and automatic process reap fallback.
   - Documents the **64KB maximum configuration file size cap** preventing denial-of-service memory exhaustion.
   - Documents the **512-byte diagnostic telemetry clamping** with `[TRUNCATED]` markers preventing audit log inflation.
2. **Fail-Closed Security Gating**:
   - Clearly documents the requirement for active cryptographic PEP grant tokens for any state-mutating commands (`aios.toolchain.set`).
3. **Lossless Error Diagnostics**:
   - Explains how missing binaries, version mismatches, and parsing errors serialize into `outcome_detail` in the SQLite WAL audit ring.

## 3. Acceptance Verification
- All failure modes, boundaries, and limits are transparently documented in `docs/README.md`.
