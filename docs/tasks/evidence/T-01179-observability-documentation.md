# T-01179: Base Image Build Observability Documentation

## Overview & Scope
Task `T-01179` documents the Base Image Build Observability subsystem (`code/aiosh-rust/aiosh-core/src/base_image_observability.rs`), including report data contracts, invariant validation (`OB1..OB5`), CLI integration (`aiosh image report`), MCP tool interface (`aios.image.report`), hardening constraints, and operational limitations.

## Subsystem Details

### Architecture & Module Location
- **Core Module**: `code/aiosh-rust/aiosh-core/src/base_image_observability.rs`
- **CLI Subcommand**: `code/aiosh-rust/aiosh-cli/src/main.rs` (`aiosh image report`)
- **MCP Tool**: `code/aiosh-rust/aiosh-mcp/src/main.rs` (`aios.image.report`)
- **Unit & Property Tests**: `code/aiosh-rust/aiosh-core/tests/test_base_image_observability.rs`

### Data Contract: `BaseImageObservabilityReport`
The report structure aggregates:
1. `generated_at`: ISO 8601 UTC timestamp of report generation.
2. `total_images`: Total count of valid image manifests in the store.
3. `format_breakdown`: Map of target format (e.g. `raw`, `qcow2`, `iso`) to image count.
4. `architecture_breakdown`: Map of target architecture (e.g. `x86_64`, `aarch64`) to image count.
5. `distro_breakdown`: Map of Linux distribution ID (e.g. `debian`, `arch`, `alpine`) to image count.
6. `policy_compliant_count`: Count of images passing the security policy in `Enforcing` mode.
7. `total_size_budget_bytes`: Sum of `size_budget_bytes` across all registered manifests.
8. `average_size_budget_bytes`: Integer average size budget (`total_size_budget_bytes / total_images`, or 0 when empty).
9. `unique_kernel_versions`: Deduplicated sorted list of unique kernel versions referenced.

### Mathematical Invariants (OB1..OB5)
- **OB1**: `sum(format_breakdown.values()) == total_images`
- **OB2**: `sum(architecture_breakdown.values()) == total_images`
- **OB3**: `sum(distro_breakdown.values()) == total_images`
- **OB4**: `policy_compliant_count <= total_images`
- **OB5**: `average_size_budget_bytes == (total_images > 0 ? total_size_budget_bytes / total_images : 0)`

### Copy-Pasteable Invocations

#### CLI Invocation
Human-readable table view:
```bash
aiosh image report --store /var/lib/aios/images
```

Machine-readable JSON envelope:
```bash
aiosh image report --json --store /var/lib/aios/images
```

Example JSON output:
```json
{
  "code": 0,
  "data": {
    "generated_at": "2026-09-04T06:00:00Z",
    "total_images": 2,
    "format_breakdown": {
      "raw": 2
    },
    "architecture_breakdown": {
      "x86_64": 2
    },
    "distro_breakdown": {
      "debian": 2
    },
    "policy_compliant_count": 2,
    "total_size_budget_bytes": 4294967296,
    "average_size_budget_bytes": 2147483648,
    "unique_kernel_versions": [
      "6.6.137-aios-standard"
    ]
  },
  "error": null
}
```

#### MCP Invocation
Call via JSON-RPC 2.0 tool execution:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "aios.image.report",
    "arguments": {
      "store_path": "tests/fixtures/images"
    }
  }
}
```

Response envelope:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"average_size_budget_bytes\":2147483648,\"total_images\":2,...}"
      }
    ]
  }
}
```

### Hardening & Resource Limits
1. **Format Capacity**: Maximum 16 unique format strings.
2. **Architecture Capacity**: Maximum 64 unique architectures.
3. **Distribution Capacity**: Maximum 256 unique distribution keys.
4. **Kernel Version Capacity**: Maximum 256 unique kernel versions.
5. **Character Sanitization**: Breakdown keys must consist solely of printable ASCII characters; control characters (`< 0x20` or `0x7f`) or null bytes cause immediate rejection with code `INVALID_ARGUMENT`.
6. **Audit Integration**: Every CLI and MCP report query writes an immutable record to the SQLite WAL audit ring with execution parameters and timing.

### Known Limitations
1. **Synchronous Execution**: Aggregation reads from the local filesystem store sequentially. Registries with tens of thousands of image manifests should be partitioned or cached.
2. **Disk I/O Bounds**: Dependent on the OS filesystem cache and JSON deserialization throughput.
3. **Historical Metrics**: Does not retain time-series history in memory; historical tracking relies on log shipping or telemetry scraping of emitted audit records.

## Related Evidence & Prior Tasks
- `docs/tasks/evidence/T-01171-base-image-observability-research.md`: Observability research and requirements
- `docs/tasks/evidence/T-01172-base-image-observability-specification.md`: Formal specification of report schema & invariants
- `docs/tasks/evidence/T-01173-base-image-observability-scaffold.md`: Interface skeleton and module declarations
- `docs/tasks/evidence/T-01174-base-image-observability-implementation.md`: Core aggregation logic implementation
- `docs/tasks/evidence/T-01175-base-image-observability-unit-tests.md`: Test suite with invariant validation
- `docs/tasks/evidence/T-01176-base-image-observability-integration.md`: CLI & MCP tool wiring
- `docs/tasks/evidence/T-01177-base-image-observability-security.md`: Threat modeling and abuse analysis
- `docs/tasks/evidence/T-01178-base-image-observability-hardening.md`: Capacity bounds and control character defenses
