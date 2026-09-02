# T-00319 — Dependency & Toolchain Pinning: Data Model Documentation

## Overview
We documented the Dependency & Toolchain Pinning `ToolchainManifest` feature natively within the `docs/README.md` file, providing operators and agents with usage examples and clear limitations.

## Documentation Highlights
- **Configuration Semantics**: Documented how `$AIOSH_TOOLCHAIN_CONFIG` overrides the default `config/toolchain.json` search path, and documented the expected schema fields.
- **MCP Surface**: Added the JSON signature required to query the active toolchain via `aios.toolchain.config.get`.
- **Honest Limitations**: Explicitly noted that:
  - The governance is currently passive (data model verification only), awaiting kernel boundary enforcement in a future service phase.
  - The configuration file size is strictly capped at 64KB.
  - Node version enforcement is optional to support headless/slim deployment environments.

## Verification
- README.md was successfully updated in place and passes visual review.
- All evidence links trace cleanly to `tasks/evidence/`.
