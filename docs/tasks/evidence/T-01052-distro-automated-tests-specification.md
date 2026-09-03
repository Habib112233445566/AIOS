# T-01052 — Distro Selection & Justification / Automated Tests: Specification

**Date:** 2026-09-03
**Type:** Specification
**Subsystem:** Phase 1 — Linux Base System & Bootable Target
**Component:** Distro Selection & Justification / Automated Tests

## 1. Specification of Test Suite Extensions

### A. Criterion D5: Configuration Subsystem (`tools/test_distro_suites.py`)
```python
def test_d5_configuration_subsystem():
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(ROOT / "code" / "aiosh-rust" / "Cargo.toml"),
        "--lib",
        "distro_config::tests",
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, cwd=str(ROOT), timeout=120)
    if res.returncode != 0:
        print(f"[-] D5 cargo test failed:\n{res.stderr}\n{res.stdout}", file=sys.stderr)
        return False
    print("[+] D5 distro configuration resolution & hardening invariants")
    return True
```

### B. Unit Assertions U08..U10 (`tools/test_distro_unit.py`)
- **U08**: `test_d5_configuration_subsystem` function exists and is callable.
- **U09**: `test_d5_configuration_subsystem` executes and passes.
- **U10**: `aiosh distro config` CLI smoke check runs with clean exit code 0.

## 2. Invariants & Exit Codes
- All runners must be self-contained, using only Python stdlib.
- Main runner returns 0 on all criteria passing, 1 on any failure.
