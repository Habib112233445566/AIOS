# T-01007 — Distro Selection & Justification / Data Model: Security Review

## 1. Threat Modeling & Abuse Scenarios

### AS-1: Insecure Profile ID Injection
- **Threat**: Malicious actor provides custom profile ID containing path traversal `../` or shell metacharacters to compromise rootfs build paths.
- **Mitigation**: `validate_distro_profile` strictly enforces alphanumeric, hyphen, and underscore characters only (`c.is_ascii_alphanumeric() || c == '-' || c == '_'`).

### AS-2: Kernel Version Spoofing
- **Threat**: Specifying an ancient or malformed kernel version that lacks cgroup v2 or Landlock LSM support.
- **Mitigation**: `validate_distro_profile` verifies numeric semver components, and `DistroEvaluation` checks minimum kernel requirement >= 6.1 LTS.

### AS-3: Unvetted External Packages
- **Threat**: Injecting arbitrary package names into default_packages list.
- **Mitigation**: Profiles are strongly typed and validated against canonical distribution package manifests.
