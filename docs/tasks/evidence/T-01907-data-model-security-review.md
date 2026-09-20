# Task Evidence: T-01907 - System Update Mechanism / data model: Security Review

## 1. Overview
- **Task ID**: `T-01907`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Perform comprehensive security review and threat modeling of the System Update Mechanism data structures, cryptographic validation rules, and slot state invariants.

---

## 2. Threat Modeling & Vulnerability Analysis

| Threat ID | Threat Category | Description | Severity | Target Mitigation |
|---|---|---|---|---|
| `THREAT-UPD-01` | Directory Traversal | Malicious artifact `file_name` containing `..`, path separators (`/`, `\`), or control characters to write outside update staging directory. | **HIGH** | Strict filename sanitization: reject any filename with slashes, backslashes, `..`, or non-printable ASCII characters. |
| `THREAT-UPD-02` | Digest Evasion | Malformed, truncated, or non-hexadecimal SHA-256 strings evading cryptographic verification. | **HIGH** | Exact 64-character lowercase/uppercase ASCII hex validation and normalization. |
| `THREAT-UPD-03` | Downgrade / Replay Attack | Replay of old, signed manifests to regress system to a vulnerable version. | **HIGH** | Version constraint checking and `min_version` validation; enforcing monotonically non-decreasing releases. |
| `THREAT-UPD-04` | Active Slot Mutation | Writing update payloads into the currently running/active boot partition (`current_slot == target_slot`), bricking the running system. | **CRITICAL** | Strict invariant check (`UPD1`): target slot MUST be strictly opposite (`current_slot.other()`). |
| `THREAT-UPD-05` | Denial-of-Service (DoS) | Unbounded artifact list count or integer overflow when calculating total payload byte count. | **MEDIUM** | Bound artifact count (e.g. max 32 artifacts) and use checked/saturating addition for `total_bytes()`. |
| `THREAT-UPD-06` | State Machine Desync | Skipping critical verification steps (e.g. jumping directly from `Idle` or `Downloading` to `ReadyToReboot`), bypassing signature or digest checks. | **CRITICAL** | Linear transition enforcement (`UPD4`), rejecting illegal jumps with `UPD_STATE_ERROR`. |

---

## 3. Required Hardening Actions for T-01908
1. **Filename Path Hygiene**: In `UpdateArtifact::validate()`, reject filenames containing `/`, `\`, `..`, leading/trailing spaces, or control characters. Limit filename length to 128 characters.
2. **Artifact Count Bound**: In `UpdateManifest::validate()`, enforce a ceiling of `MAX_ARTIFACTS = 32` artifacts per manifest.
3. **Integer Overflow Guard**: Replace raw `.sum()` in `UpdateManifest::total_bytes()` with `saturating_add` or checked summation, ensuring safe computation even with edge-case artifact declarations.
4. **Digest Normalization**: Convert SHA-256 to lowercase ASCII hex during validation for canonical consistency.

---

## 4. Verification
Threat modeling and mitigation specifications formulated; ready for hardening implementation in `T-01908`.
