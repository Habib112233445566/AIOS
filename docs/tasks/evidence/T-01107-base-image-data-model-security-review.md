# T-01107 — Base Image Build / Data Model: Security Review

**Date:** 2026-09-03
**Type:** Security Review
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Base Image Build / Data Model

## 1. Security Review Analysis
- **Package Name Injection**: Reviewed `rootfs.packages`. If package names contain spaces, newlines, or command separators (`;`, `&`, `|`), downstream invocations of `debootstrap` or package managers could suffer argument or shell injection.
- **Hostname Injection**: Reviewed `rootfs.hostname`. Hostnames must strictly adhere to RFC 1123 conventions to prevent `/etc/hostname` poisoning.
- **Kernel Cmdline Sanitization**: Checked `kernel.cmdline` to ensure prohibition of control characters and null bytes.
- **Resource Exhaustion Bounds**: Confirmed `size_budget_bytes` ceiling at 10 GiB.

## 2. Hardening Directives for T-01108
- Add package name character validation: each package name must match `^[a-z0-9+.-]+$`.
- Add RFC 1123 hostname validation: maximum 63 characters, lowercase alphanumeric and hyphens, no trailing/leading hyphen.
- Reject control characters (CR, LF, NUL) in `kernel.cmdline`.
