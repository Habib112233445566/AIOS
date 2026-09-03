# T-01047 — Distro Selection & Justification / Configuration: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Configuration

## 1. Security Analysis & Threat Posture
- **Input Boundaries**: File reads capped at 65,536 bytes (`take(MAX_DISTRO_CONFIG_BYTES)`). Protects against unbounded memory consumption and file descriptor exhaustion.
- **Floating-Point Sanitization**: Evaluated behavior of `f32` inputs. Checked for `NaN` and `Infinity` risks in weights and recommendation threshold.
- **Path Isolation**: Environment variable values (`AIOSH_DISTRO_STORE_PATH`, `AIOSH_DISTRO_CONFIG`) cannot escape bounded paths when executing under sandboxed Landlock environments.
- **Fail-Safe Recovery**: Any malformed configuration results in explicit `Result::Err` or clean fallback to `DistroConfig::default()` without panicking.

## 2. Findings & Recommendations for Hardening (T-01048)
1. Add explicit `is_nan()` checks in `DistroConfig::validate()` to ensure IEEE 754 NaN values are strictly rejected.
2. Ensure path traversal patterns (`..`) in `store_path` are rejected if configured relative to workspace roots.
3. Add unit test asserting `NaN` rejection.
