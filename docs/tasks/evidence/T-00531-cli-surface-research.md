# T-00531 — Evidence & Audit Trail / CLI surface: Research

## 1. Goal
Establish facts, constraints, and prior art for the CLI surface of Evidence & Audit Trail in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Codebase & Architecture):
1. **Unified CLI Binary**: The `aiosh` binary (`code/aiosh-rust/aiosh-cli/src/main.rs`) is the entry point for all command execution.
2. **Dual-Mode Output**:
   - Human-readable prose mode with status indicators (`[+]`, `[-]`).
   - Machine-readable `--json` mode returning standardized JSON envelopes with `"ok": bool`, `"subcommand": str`, and `"data"` payload.
3. **Exit Code Conventions**:
   - `0`: Success / valid verification.
   - `1`: Functional failure (e.g. hash mismatch, missing evidence file).
   - `2`: Usage error / missing required arguments.
4. **Audit Integration**:
   - Every CLI execution writes a structured row into the SQLite WAL audit ring with deterministic SHA-256 chaining.

### Assumptions:
1. `aiosh evidence` should provide `verify`, `hash`, and `scan` subcommands.
2. A Python smoke test `code/aiosh-cli/tests/test_evidence_cli_smoke.py` will validate CLI execution in both human and JSON modes.

## 3. Prior Art & Authoritative Sources
- **GNU coreutils `sha256sum`**: Standard hash computation and manifest checking (`-c` flag).
- **in-toto CLI (`in-toto-verify`)**: Verification of step attestations and materials.
- **POSIX Utility Conventions (IEEE Std 1003.1)**: Exit code standardization and option parsing.

## 4. Decisions Needed
1. **Subcommands**:
   - `aiosh evidence verify [--repo <path>] [--manifest <path>] [--json]`
   - `aiosh evidence hash <path> [--json]`
   - `aiosh evidence scan [--repo <path>] [--task <id>] [--json]`
2. **Smoke Test Placement**: `code/aiosh-cli/tests/test_evidence_cli_smoke.py`.

## 5. Next Steps
Advance to Specification (T-00532) to formalize CLI syntax, arguments, flag schemas, and JSON outputs.
