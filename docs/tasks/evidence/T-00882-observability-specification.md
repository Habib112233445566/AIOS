# T-00882 — Regression Triage / Observability: Specification

## 1. Observability Data Contract

```rust
impl TriageReport {
    /// Returns breakdown by lifecycle status: (untriaged, triaged, fix_pending, resolved, wont_fix).
    pub fn status_counts(&self) -> (usize, usize, usize, usize, usize);

    /// Returns breakdown by severity: (blocker, critical, major, minor).
    pub fn severity_counts(&self) -> (usize, usize, usize, usize);

    /// Formats a standardized human-readable single-line summary string.
    pub fn summary_line(&self) -> String;
}
```

## 2. Invariants & Output Guarantees
- `sum(status_counts) == self.total_records`.
- `sum(severity_counts) == self.total_records`.
- `summary_line()` format is deterministic across platforms and locales.
