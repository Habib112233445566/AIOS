//! AIOS core library — the Rust rewrite of the aiosh userspace stack.
//!
//! This crate replaces the previous dual-substrate implementation
//! (`code/aiosh-cli` in TypeScript + `code/aiosh-mcp` in Python) with a
//! single Rust implementation. All invariants that used to be enforced
//! by a TS↔Python cross-substrate test now live here as Rust unit and
//! integration tests:
//!
//!   - canonical JSON (byte-identical to the old Python `json.dumps(
//!     sort_keys=True, separators=(",",":"))` and TS `canonicalJson`)
//!   - SHA-256 hash-chained append-only audit ring (SQLite WAL)
//!   - Constitution rule-pack classifier (R-01..R-12)
//!   - PEP grant store and gate
//!   - Sprint-3 retention (checkpointed segment rotation + bloom filter)
//!   - Pillar-A pentest wrappers with safe defaults
//!   - Landlock + seccomp-bpf sandbox for `aiosh run`
//!   - Sprint-2 agent loop (Ollama + deterministic stub)
//!   - Task Ledger Control data model (T-00014 port of
//!     `tools/task_ledger.py`: atomic state, event log, no-skip law)

pub mod agent;
pub mod audit;
pub mod canonical;
pub mod ci;
pub mod ci_config;
pub mod classifier;
pub mod dispatch;
pub mod ledger;
pub mod ledger_config;
pub mod pentest;
pub mod pep;
pub mod retention;
pub mod sandbox;
pub mod task_service;
pub mod types;

pub use audit::{AuditRing, OpenOptions};
pub use classifier::classify;
pub use pep::PepStore;
pub use types::GENESIS_HASH;
