# T-01029 — Distro Selection & Justification / CLI Surface: Documentation

## 1. CLI Usage & Documentation
Updated `docs/README.md` §8.10 with full CLI commands and smoke testing instructions.

### Syntax & Subcommands
```bash
aiosh distro list [--json] [--store <path>]
aiosh distro show <id> [--json] [--store <path>]
aiosh distro evaluate [<id>] [--json] [--store <path>]
aiosh distro recommend [--json] [--store <path>]
```

### Copy-Pasteable Verification Commands
```bash
# Display formatted table of distributions
code/aiosh-rust/target/debug/aiosh.exe distro list

# Retrieve JSON specification of recommended distro
code/aiosh-rust/target/debug/aiosh.exe distro recommend --json

# Run Python smoke test runner
python code/aiosh-cli/tests/test_distro_cli_smoke.py
```

## 2. Honest Constraints & Limitations
- Custom stores loaded via `--store` must be valid JSON adhering to `DistroStore` schema and under the 10 MiB limit (`MAX_STORE_BYTES`).
- Downstream ISO creation and packaging is handled by upcoming Phase 1 rootfs/bootstrap tasks.
