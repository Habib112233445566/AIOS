# T-02127: Security Review — PEP Decision Engine CLI Surface

## Overview
- **Task ID**: `T-02127`
- **Sub-Epic**: 3 (CLI Surface)
- **Status**: Completed

## Threat Analysis & Abuse Scenarios

### 1. Terminal Escape Sequence Injection
- **Threat**: An attacker supplies malicious ANSI escape sequences in rule IDs, descriptions, or error arguments to overwrite terminal contents or execute commands via terminal emulator vulnerabilities.
- **Verification**: All dynamic strings written to `eprintln!` are filtered through `sanitize_terminal(...)`.
- **Status**: MITIGATED.

### 2. Path Traversal via `--store` Argument
- **Threat**: An operator or script passes a traversal path (e.g. `--store /var/data/../../etc/pep_policies.json`) to overwrite sensitive files.
- **Verification**: `validate_pep_service_path` inspects every path component and rejects `std::path::Component::ParentDir`.
- **Status**: MITIGATED.

### 3. Resource Traversal Evasion in `evaluate` and `rule-add`
- **Threat**: Passing `fs:/safe/../../etc/shadow` to bypass resource prefix matching.
- **Verification**: `PepRequest::new` explicitly rejects any resource containing `..` with exit code `2`.
- **Status**: MITIGATED.

### 4. Rule ID and Parameter Injection
- **Threat**: Submitting control characters or excessively large strings in rule IDs to corrupt the JSON store.
- **Verification**: Rule IDs are checked for `id.len() > 128` and control characters; invalid IDs return exit code `2`.
- **Status**: MITIGATED.

### 5. Audit Row Emission Guarantee
- **Threat**: CLI commands failing silently or skipping audit emission.
- **Verification**: All code paths in `cmd_pep` call `classify_and_emit` with appropriate status, command name, and target before exiting.
- **Status**: MITIGATED.

## Finding Summary
Zero open vulnerabilities found. All invariants verified.
