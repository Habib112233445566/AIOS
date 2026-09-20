# T-01829: Network Bootstrap / CLI Surface: Documentation

## 1. Overview
- **Task ID**: `T-01829`
- **Sub-Epic**: 3 (CLI Surface)
- **Goal**: Document the CLI surface of Network Bootstrap for operators and agents.

---

## 2. Documentation Updates
- Updated `docs/network_bootstrap.md` with **Section 6: Network Bootstrap CLI Surface (`aiosh net` / `aiosh network`)**.
- Documented:
  - Full subcommand reference (`list`, `show <iface>`, `routes`, `dns`, `state`, `up <iface>`, `down <iface>`).
  - Flag options (`--sysfs`, `--procfs`, `--resolv`, `--json`).
  - Security and architectural invariants `NCLI1..NCLI6`.
  - Copy-pasteable terminal invocations for human and JSON modes.
  - Known constraints and limitations (e.g. Linux kernel dependencies for live discovery, `CAP_NET_ADMIN` requirements for link mutations, IPv4 route scope).

## 3. Cross-References & Evidence Links
- Research: [T-01821-cli-surface-research.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01821-cli-surface-research.md)
- Specification: [T-01822-cli-surface-specification.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01822-cli-surface-specification.md)
- Scaffold: [T-01823-cli-surface-scaffold.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01823-cli-surface-scaffold.md)
- Implementation: [T-01824-cli-surface-implementation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01824-cli-surface-implementation.md)
- Unit Tests: [T-01825-cli-surface-unit-test.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01825-cli-surface-unit-test.md)
- Integration Smoke: [T-01826-cli-surface-integration.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01826-cli-surface-integration.md)
- Security Review: [T-01827-cli-surface-security-review.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01827-cli-surface-security-review.md)
- Hardening: [T-01828-cli-surface-hardening.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01828-cli-surface-hardening.md)
