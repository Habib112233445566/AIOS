# T-01359: Init & Service Supervision - Automated Tests: Documentation

## Metadata
- **Task ID:** `T-01359`
- **Subsystem:** Phase 1 — Linux Base System & Bootable Target
- **Component:** Init & Service Supervision / Automated Tests Documentation
- **Status:** Complete
- **Date:** 2026-09-06

---

## 1. Documentation Overview

Documented the Init & Service Supervision Automated Test Suite (`code/aiosh-rust/aiosh-core/tests/test_service_automated.rs`) and test harness (`tools/test_service_suites.py`) in `docs/README.md`.

### Delivered Test Capabilities:
- **Suite Criteria (`ST1..ST5`)**:
  - `ST1`: Multi-turn lifecycle FSM transitions and security masking refusal.
  - `ST2`: Kahn's topological sort dependency resolution and cyclic dependency aborts.
  - `ST3`: Atomic store serialization, crash recovery, and status persistence.
  - `ST4`: Configuration quota enforcement and size bounds.
  - `ST5`: Filtered querying and catalog introspection.
- **Subsystem Test Runner Criterion (`SS6`)**:
  - Registered `SS6` in `tools/test_service_suites.py`.

---

## 2. Copy-Pasteable Usage Examples

### Running the Automated Integration Test Suite in Isolation:
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_service_automated
```

### Running the Subsystem Test Runner (`SS1..SS6`):
```bash
python tools/test_service_suites.py
```

Expected Output:
```
[+] SS1 service data model integrity & invariants (SS1..SS5)
[+] SS2 service CLI surface commands & options (validate, list, show/status, action, order)
[+] SS3 service MCP tool surface (validate, list, get, action, order, config)
[+] SS4 service core service lifecycle, FSM & dependency ordering (CS1..CS5)
[+] SS5 service configuration subsystem invariants, precedence & sizing (SC1..SC7)
[+] SS6 service automated integration tests (ST1..ST5)

PASS: service_suites criteria (SS1..SS6)
```

---

## 3. Constraints & Known Limitations (Honest)
1. **Subprocess Timeout Ceiling**: All test invocations through `tools/test_service_suites.py` are subject to a strict 120-second timeout to prevent deadlocks.
2. **Process Privilege Scope**: Tests run in non-root userspace by virtualizing service records, PIDs, and stores in JSON rather than spawning actual kernel processes requiring root.
3. **Dependency Cycle Detection**: Circular dependency detection operates on static `Requires` and `After` graphs; dynamic runtime circular dependencies must be detected prior to service activation.

---

## 4. Linked Evidence Chain
- Research: `docs/tasks/evidence/T-01351-automated-tests-research.md`
- Specification: `docs/tasks/evidence/T-01352-automated-tests-specification.md`
- Scaffold: `docs/tasks/evidence/T-01353-automated-tests-scaffold.md`
- Implementation: `docs/tasks/evidence/T-01354-automated-tests-implementation.md`
- Unit Tests: `docs/tasks/evidence/T-01355-automated-tests-unit-test.md`
- Integration: `docs/tasks/evidence/T-01356-automated-tests-integration.md`
- Security Review: `docs/tasks/evidence/T-01357-automated-tests-security-review.md`
- Hardening: `docs/tasks/evidence/T-01358-automated-tests-hardening.md`
- Documentation: `docs/tasks/evidence/T-01359-automated-tests-documentation.md`
