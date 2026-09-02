# T-00657 — Repository Health / configuration: Security Review

## 1. Security Review Scope
This task evaluates the security posture and input handling of the `RepoHealthConfig` subsystem against path traversal, denial-of-service, and threshold manipulation threats.

## 2. Threat Model & Abuse Scenarios

### Scenario CFG-1: Oversized Config Files & Memory Exhaustion
- **Threat**: An adversary provides a multi-gigabyte config file to cause out-of-memory crashes.
- **Finding & Mitigation**:
  - `RepoHealthConfig::from_path` limits file consumption to `MAX_CONFIG_BYTES` (64 KiB) using `.take(MAX_CONFIG_BYTES)`.
  - Config reading cannot exceed 64 KiB regardless of file size on disk.

### Scenario CFG-2: Path Traversal in Path Fields
- **Threat**: Injecting relative directory escapes (`../../sensitive_dir`) in `ignored_dirs` or `security_policy_path`.
- **Finding & Mitigation**:
  - `validate()` explicitly forbids `..`, `/`, and `\` in `ignored_dirs`.
  - `validate()` explicitly forbids `..` in `security_policy_path`.
  - Any traversal attempt triggers a hard validation error before use.

### Scenario CFG-3: Extreme/Zero Boundary Threshold Manipulation
- **Threat**: Supplying `max_file_bytes = 0` or negative values to force immediate check failure.
- **Finding & Mitigation**:
  - `validate()` checks `1024 <= max_file_bytes <= 1073741824`.
  - `min_security_policy_bytes` is bounded between `1` and `65536`.

## 3. Verdict
- **Status**: PASS
- **Open Vulnerabilities**: 0
- **Residual Risks**: None identified.
