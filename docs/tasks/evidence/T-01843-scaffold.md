# Scaffold Evidence - T-01843: Network Bootstrap Configuration Scaffold

- Module: `code/aiosh-rust/aiosh-core/src/network_config.rs`
- Crate Root Re-export: `code/aiosh-rust/aiosh-core/src/lib.rs` (`pub mod network_config; pub use network_config::NetworkConfig;`)
- Invariants targeted: `NCONF1..NCONF6`
- Compilation check: `cargo check --manifest-path code/aiosh-rust/Cargo.toml -p aiosh-core` passed cleanly.
