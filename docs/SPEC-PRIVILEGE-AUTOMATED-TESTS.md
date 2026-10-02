# Privilege Escalation Prevention Automated Tests Guide

- **Status**: ACTIVE
- **Subsystem**: `aiosh-core`, `aiosh-cli`, `aiosh-mcp`
- **Updated**: 2026-09-29

## 1. Overview
The Privilege Escalation Prevention automated test suite ensures that all privilege boundaries, capability assignments, and state transitions are strictly governed by PEP policy and immutable security invariants.

## 2. Test Suites & Execution
### Rust Automated Suite
Runs the formal automated test vectors in `code/aiosh-rust/aiosh-core/tests/test_privilege_automated.rs`:
```bash
cargo test --manifest-path code/aiosh-rust/Cargo.toml --test test_privilege_automated
```

Vectors tested:
- `AUTOPRIV1`: Multi-tenant context lifecycle & state isolation
- `AUTOPRIV2`: SystemKernel tier immutability & fail-closed denial
- `AUTOPRIV3`: Token validation, replay prevention, and grant gating
- `AUTOPRIV4`: Dynamic capability granting, check, and attenuation
- `AUTOPRIV5`: Privilege drop, revoke, and baseline restoration
- `AUTOPRIV6`: Store persistence, roundtrip serialization, and size bounding
- `AUTOPRIV7`: Context unregister & capacity overflow boundaries
- `AUTOPRIV8`: Multi-threaded concurrency safety under concurrent transitions
- `AUTOPRIV9`: Corrupted and malformed state resilience

### Python CLI Integration Suite
```bash
python code/aiosh-cli/tests/test_privilege_cli.py
python code/aiosh-cli/tests/test_privilege_automated.py
```

### Python MCP Integration Suite
```bash
python code/aiosh-mcp/tests/test_privilege_mcp.py
python code/aiosh-mcp/tests/test_privilege_automated_smoke.py
```
