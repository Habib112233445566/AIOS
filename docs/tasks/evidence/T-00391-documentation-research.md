# T-00391 — Dependency & Toolchain Pinning / documentation: Research

## 1. Goal
Establish facts, constraints, user/agent documentation requirements, and prior art for the documentation of Dependency & Toolchain Pinning in AIOS.

## 2. Facts vs. Assumptions

### Facts (Empirical from Current Code & Documentation):
1. **Documentation Invariants**: `tools/check_task_docs.py` enforces structural documentation criteria (C1..C6) on `docs/README.md` and related spec files.
2. **Dual-Audience Requirements**:
   - *Human Operators*: Require CLI syntax (`aiosh toolchain show`, `aiosh toolchain check`), environment variables (`$AIOSH_TOOLCHAIN_CONFIG`), and exit code conventions.
   - *Autonomous Agents*: Require exact MCP tool definitions (`aios.toolchain.config.get`, `aios.toolchain.check`), JSON-RPC schemas, provenance metadata interpretations, and error refusal formats.
3. **Supported Toolchain Pin Files**: The repo maintains root toolchain anchors:
   - `config/toolchain.json` (canonical JSON manifest)
   - `rust-toolchain.toml` (Rust compiler channel pin)
   - `.python-version` (Python runtime pin)

### Assumptions:
1. Clear, copy-pasteable examples for both CLI and MCP surfaces reduce agent hallucination during task execution.
2. Explicitly stating known limitations (subprocess timeouts, 64KB file caps) avoids operator confusion in airgapped or restricted environments.

## 3. Prior Art & Authoritative Standards
- **Diátaxis Documentation Framework**: Separation of how-to guides (running checks), reference (JSON schema and MCP arguments), and explanation (why hermetic pinning matters).
- **Rust Toolchain Documentation (rustup)**: `rust-toolchain.toml` format and channel resolution.
- **PEP 511 / pyenv**: `.python-version` format conventions.

## 4. Decisions Needed
1. **Structure Consolidation**: Consolidate CLI, MCP, Security, and Observability subsections in `docs/README.md` into a single, cohesive developer/operator manual.
2. **Verification Suite Inclusion**: Document exact test runner invocations for verifying toolchain health locally.

## 5. Next Steps
Advance to Specification (T-00392) to define the exact layout, examples, and schema for the documentation deliverables.
