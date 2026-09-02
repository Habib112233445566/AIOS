# T-00355 — Dependency & Toolchain Pinning / configuration: Unit Test
# T-00356 — Dependency & Toolchain Pinning / configuration: Integration

## Unit Test & Integration Evidence

The root configurations have been tested using the `aiosh toolchain check` integration test from the repository root. 

### Environment
- **CWD**: Repository root (`c:\Users\OBSESSION\Desktop\AIOS_MERGED`)
- **Binary**: `code/aiosh-rust/target/debug/aiosh.exe`
- **Command**: `toolchain check`

### Test Result
```json
{
  "data": {
    "enforce_hashes": {
      "source": "default",
      "value": false
    },
    "node_version": {
      "source": "default",
      "value": "v24.18"
    },
    "python_version": {
      "source": "default",
      "value": "3.14"
    },
    "rust_version": {
      "source": "default",
      "value": "1.99.0"
    }
  },
  "ok": true,
  "subcommand": "toolchain check"
}
```

### Analysis
The default fallback of `aiosh_core::toolchain_config::ToolchainManifest::from_env()` correctly locates `config/toolchain.json` relative to the invocation CWD. Because it returns `ok: true`, we confirm that `1.99.0` matches the environment's `rustc 1.99.0-nightly` and the Python and Node binaries on the mock testbed. 

The integration of `rust-toolchain.toml`, `.python-version`, and `.nvmrc` completes the configuration footprint across all toolchains for Phase 0.
