# T-01179: Base Image Observability Documentation (Alias)

See [T-01179-observability-documentation.md](file:///c:/Users/OBSESSION/Desktop/AIOS_MERGED/docs/tasks/evidence/T-01179-observability-documentation.md) for the complete documentation artifact.

## Highlights
- **Module**: `code/aiosh-rust/aiosh-core/src/base_image_observability.rs`
- **CLI Subcommand**: `aiosh image report [--json] [--store <path>]`
- **MCP Tool**: `aios.image.report`
- **Invariants**: `OB1..OB5`
- **Hardening**: Capacities bounded (16 formats, 64 archs, 256 distros, 256 kernels), control character rejection, and SHA-256 audit chaining.
