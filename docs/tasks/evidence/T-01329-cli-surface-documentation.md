# T-01329: Init & Service Supervision - CLI Surface: Documentation

## Metadata
- **Task ID:** `T-01329`
- **Subsystem:** `code/aiosh-rust/aiosh-cli`
- **Component:** Init & Service Supervision CLI Surface Documentation
- **Status:** Complete

## 1. Documentation Deliverables
- Updated `docs/README.md` §8.13 with complete command syntax, parameters, options, and operational guidelines for `aiosh service`.
- Documented all core subcommands (`validate`, `list`, `show`, `status`, `action`, `order`) and direct shortcuts (`start`, `stop`, `restart`, `reload`, `enable`, `disable`, `mask`, `unmask`).
- Documented standard JSON envelopes and exit code contracts.
- Updated test runner outputs and evidence references in the documentation suite.

## 2. Copy-Pasteable Usage Examples
```bash
# 1. Validate service name syntax (SS1)
aiosh service validate --name aios-securityd.service

# 2. List active services with formatted JSON output
aiosh service list --state active --json

# 3. Inspect service metadata using the status alias
aiosh service status auditd.service

# 4. Execute direct lifecycle action shortcut
aiosh service restart auditd.service

# 5. Plan topological startup sequence for a service dependency chain
aiosh service order aios-securityd.service
```

## 3. Honest Limitations & Constraints
- **Scope**: The CLI surface manipulates the in-memory/JSON persistent service store (`ServiceStore`); kernel-level process orchestration (`cgroups`, namespaces, PID 1 execution) is managed by downstream init daemon runners.
- **Payload Limits**: Specification inputs (`--spec`) are bounded at 1 MiB. Service names are capped at 128 characters.
- **Store Path**: Overriding the store path (`--store <path>`) requires an existing store file unless created through programmatic store seeding.
