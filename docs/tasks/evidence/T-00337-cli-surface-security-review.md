# T-00337 — Dependency & Toolchain Pinning / CLI surface: Security Review

## 1. New Surface Since T-327

### `aiosh toolchain show`
- **Read-only**: Calls `from_env()` or `from_path()` and outputs the manifest. No state changes, no enforcement. No PEP grant required.
- **Audit row**: Emitted on both success and error paths.
- **Risk**: None beyond what was already covered in T-327 for `check`.

### `--config <path>` flag
- **Input validation**: The path is passed directly to `std::fs::File::open()`, which is bound by the process's OS permissions. No shell interpolation or injection risk.
- **Path traversal**: An operator can point `--config ../../../etc/passwd` but `serde_json::from_str` will reject it as invalid JSON. The error message will contain the parse error, not the file contents (since the error is from serde, not from reading the file).
- **Size cap**: The `from_source` reader uses `.take(65_536)` regardless of how the path was provided. This prevents memory exhaustion.
- **Precedence**: `--config` overrides `$AIOSH_TOOLCHAIN_CONFIG`. This is documented behavior and is safe — the operator must have CLI access to use the flag.

## 2. Abuse Scenarios

| Scenario | Risk | Mitigation |
|---|---|---|
| `--config /dev/zero` (Linux) | Memory exhaustion | `.take(65_536)` bounds the read |
| `--config ../../../etc/shadow` | File disclosure | serde rejects as invalid JSON; error message is sanitized |
| Unknown flags | Argument injection | Unknown flags cause exit 2 immediately |
| Missing `--config` value | Hang or crash | Detected and exits 2 before file open |

## 3. Conclusion
No policy bypasses or new vulnerabilities introduced. The CLI surface extensions (`show`, `--config`) maintain the same security properties as the base `check` command reviewed in T-327.
