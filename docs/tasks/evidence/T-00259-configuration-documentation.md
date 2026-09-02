# T-00259 — Configuration: Documentation

## Documentation Updates

**File Modified**: `docs/README.md`

**Updates Made**:
1. Added a **Configuration** subsection under **Release Packaging & Backup** explaining how configuration is loaded natively in Rust.
2. Included a **copy-pasteable example** showing how to set up a custom JSON configuration file and invoke it via the `$AIOSH_RELEASE_CONFIG` environment variable.
3. Added to **Known Limitations**: Recorded the 64KB config file limit (to prevent OOM DoS) and the strict rejection of malicious `output_dir` paths.
4. Linked the config task evidence files (`T-00251` through `T-00260`) in the evidence section.

## Acceptance Validation
- The documentation accurately reflects what shipped in the configuration hardening task (`T-00258`).
- Example command is practical and ready to run.
- Constraints and known limitations are explicitly documented.
