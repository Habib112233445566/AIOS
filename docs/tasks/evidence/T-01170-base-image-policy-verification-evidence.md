# T-01170 — Base Image Build / Security Policy: Verification & Evidence

**Date:** 2026-09-04
**Type:** Verification & Evidence Closure
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Security Policy

## 1. Sub-Epic Summary (T-01161..T-01170)
- **T-01161 (Research)**: Established facts vs assumptions, threat models, and policy dimensions P1..P7.
- **T-01162 (Specification)**: Formally specified `BaseImageSecurityPolicy`, `BaseImagePolicyVerdict`, and evaluation rules.
- **T-01163 (Scaffold)**: Created module skeleton in `code/aiosh-rust/aiosh-core/src/base_image_policy.rs` and wired into `lib.rs`.
- **T-01164 (Implementation)**: Implemented complete policy evaluation algorithm, environment loading, and store filtering.
- **T-01165 (Unit Test)**: Created standalone integration/unit test suite `code/aiosh-rust/aiosh-core/tests/test_base_image_policy.rs` testing P1..P7.
- **T-01166 (Integration)**: Exposed policy surface via `aiosh image policy` CLI and `aios.image.policy` MCP tool.
- **T-01167 (Security Review)**: Conducted security analysis covering abuse scenarios and parameter obfuscation.
- **T-01168 (Hardening)**: Enforced size caps and input poisoning checks rejecting control characters and null bytes (`P0_MALFORMED_INPUT`).
- **T-01169 (Documentation)**: Updated `docs/README.md` with CLI and MCP invocation examples and invariants.
- **T-01170 (Verification)**: Full test suite verification across B1..B7.

## 2. Test Execution Output
```
[+] B1 base image data model integrity & invariant validation
[+] B2 base image store registry, persistence & build plan synthesis
[+] B3 base image CLI surface commands & options (list/show/plan/filter)
[+] B4 base image MCP surface tools (list/get/plan)
[+] B5 base image configuration invariants & precedence (CF1..CF6)
[+] B6 base image automated integration test suite (T1..T7)
[+] B7 base image security policy enforcement & invariants (P1..P7)

PASS: image_suites criteria (B1..B7)
```
