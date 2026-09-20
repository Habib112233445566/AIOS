# Task Evidence: T-01901 - System Update Mechanism / data model: Research

## 1. Overview
- **Task ID**: `T-01901`
- **Sub-Epic**: 1 (System Update Mechanism Data Model)
- **Goal**: Research data model patterns, A/B slot abstractions, payload integrity verification, and state transition invariants for System Update Mechanism.

---

## 2. Architectural Survey & Domain Patterns
1. **A/B Dual-Slot Partition Model**:
   - Modern immutable OS updates (systemd-sysupdate, Android A/B, ChromeOS, Flatcar) rely on dual bootable slots (`SlotA` and `SlotB`).
   - One slot is active/running; the update is streamed and applied to the inactive slot.
   - On successful boot verification, the bootloader flag toggles the active slot. If boot fails or health check fails within a watchdog timeout, the system automatically falls back to the previous slot.
2. **Cryptographic Payload Integrity**:
   - Image updates require strict digest pinning (SHA-256) and optional signature verification.
   - Size verification ensures partitions are not overrun and prevents resource exhaustion.
3. **State Transition Machine**:
   - Updates follow a strict unidirectional state machine:
     `Idle -> Checking -> Downloading -> Verifying -> Applying -> ReadyToReboot -> Verified` (or `RolledBack` / `Failed`).
   - Jumping states or applying unverified payloads is strictly rejected.

---

## 3. Formulated Safety Invariants (`UPD1..UPD6`)
- `UPD1` (Active Slot Exclusivity): Exactly one slot is designated `Active` at any given time. Updates are written only to the inactive slot.
- `UPD2` (Version Semantics): Target update version must be non-empty and satisfy semantic versioning (`major.minor.patch`). Downgrades require explicit flags.
- `UPD3` (Cryptographic Digest Integrity): Every update manifest requires a valid 64-character hex SHA-256 digest.
- `UPD4` (Strict State Transitions): State machine enforces sequential progression.
- `UPD5` (Rollback Readiness): System tracks previous successful boot slot to guarantee fallback availability.
- `UPD6` (Path Hygiene & JSON Parity): Manifest and image paths bounded to 1024 characters, free of `..` and control characters, with byte-for-byte canonical JSON roundtripping.

Status: Research completed. Ready for specification in `T-01902`.
