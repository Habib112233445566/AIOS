# T-00782 — Secrets & Access Hygiene / observability: Specification

## 1. Observability Contract Specification

### Metrics Data Model (`SecretScanReport`)
```rust
impl SecretScanReport {
    /// Returns a tuple of (critical, high, medium, low, info) finding counts.
    pub fn severity_counts(&self) -> (usize, usize, usize, usize, usize) { ... }
}
```

### JSON Telemetry Schema
```json
{
  "scanned_files": 42,
  "scanned_bytes": 1048576,
  "duration_ms": 15,
  "clean": true,
  "findings": []
}
```

### CLI Human-Readable Telemetry
Format:
```text
Scan finished in 15ms: 42 files scanned (1048576 bytes), 0 findings (clean: true)
```
