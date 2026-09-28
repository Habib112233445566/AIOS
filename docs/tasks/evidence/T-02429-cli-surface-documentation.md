# T-02429: Sandbox Enforcement CLI Surface Documentation

## 1. Documentation Scope
This document records evidence for task T-02429: Sandbox Enforcement CLI Surface Documentation.
The CLI surface specification was documented in [`docs/SPEC-SANDBOX-CLI.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/SPEC-SANDBOX-CLI.md) and integrated into the primary command help index.

## 2. Command Synopsis & Examples
Operators and subagents can query help and invoke commands:
```bash
# Display help
aiosh sandbox --help

# List profiles
aiosh sandbox profiles --json

# Probe host kernel capabilities
aiosh sandbox probe

# Supervised execution
aiosh sandbox exec --profile permissive -- python -c "print('hello_sandbox')"
```

## 3. Constraints & Limitations
1. **Mandatory Delimiter**: Execution commands must be separated from `aiosh` flags by `--`.
2. **Directory Traversal**: Relative parent paths (`..`) are strictly forbidden in `--cwd`.
3. **Platform Constraints**: Native kernel containment (Landlock LSM, Seccomp-BPF) is active on Linux; non-Linux hosts operate with fallback process supervision.

## 4. Evidence References
- Research: [`T-02421-cli-surface-research.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02421-cli-surface-research.md)
- Specification: [`T-02422-cli-surface-specification.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02422-cli-surface-specification.md)
- Scaffold: [`T-02423-cli-surface-scaffold.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02423-cli-surface-scaffold.md)
- Implementation: [`T-02424-cli-surface-implementation.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02424-cli-surface-implementation.md)
- Unit Testing: [`T-02425-cli-surface-unit-test.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02425-cli-surface-unit-test.md)
- Integration: [`T-02426-cli-surface-integration.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02426-cli-surface-integration.md)
- Security Review: [`T-02427-cli-surface-security-review.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02427-cli-surface-security-review.md)
- Hardening: [`T-02428-cli-surface-hardening.md`](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-02428-cli-surface-hardening.md)
