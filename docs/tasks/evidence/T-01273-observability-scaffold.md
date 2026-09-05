# T-01273: Package Management / Observability - Scaffold

## Overview
Task `T-01273` scaffolds the Rust core module for Package Management Observability:
1. Created `code/aiosh-rust/aiosh-core/src/package_observability.rs`.
2. Defined typed structure `PackageObservabilityReport` and method signature `PackageObservabilityReport::generate(store: &PackageStore, policy_opt: Option<&PackageSecurityPolicy>) -> Self`.
3. Exported `pub mod package_observability;` in `code/aiosh-rust/aiosh-core/src/lib.rs`.
4. Verified compilation across all workspace crates (`aiosh-core`, `aiosh-sandbox`, `aiosh-cli`, `aiosh-mcp`) with zero warnings or errors.

## Scaffolding Implementation
```rust
//! Observability and telemetry reports for AIOS Package Management subsystem (PO1..PO6).

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::package_service::PackageStore;
use crate::package_policy::PackageSecurityPolicy;

/// Comprehensive observability report detailing package store inventory, storage, and policy compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageObservabilityReport {
    pub total_packages: usize,
    pub state_breakdown: BTreeMap<String, usize>,
    pub format_breakdown: BTreeMap<String, usize>,
    pub architecture_breakdown: BTreeMap<String, usize>,
    pub total_installed_size_bytes: u64,
    pub average_package_size_bytes: u64,
    pub dependency_distribution: BTreeMap<String, usize>,
    pub policy_compliant_count: usize,
    pub policy_violations_count: usize,
    pub prohibited_packages_found: Vec<String>,
    pub generated_at: String,
}

impl PackageObservabilityReport {
    /// Generates an observability report from the provided PackageStore and optional security policy.
    pub fn generate(
        _store: &PackageStore,
        _policy_opt: Option<&PackageSecurityPolicy>,
    ) -> Self {
        unimplemented!("PackageObservabilityReport::generate is not implemented yet in scaffold");
    }
}
```

## Compilation Verification
`cargo check --manifest-path code/aiosh-rust/Cargo.toml`
Exit Code: 0. Clean compilation.
