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
pub mod distro;
pub mod distro_service;
pub mod doc_index;
pub mod doc_index_config;
pub mod doc_index_service;
pub mod evidence;
pub mod evidence_config;
pub mod evidence_service;
pub mod handoff;
pub mod handoff_config;
pub mod handoff_service;
pub mod ledger;
pub mod ledger_config;
pub mod pentest;
pub mod pep;
pub mod release;
pub mod release_config;
pub mod repo_health;
pub mod repo_health_config;
pub mod repo_health_service;
pub mod retention;
#[allow(dead_code)]
pub mod sandbox;
pub mod secrets;
pub mod secrets_config;
pub mod secrets_service;
pub mod task_service;
pub mod toolchain_config;
pub mod toolchain_service;
pub mod triage;
pub mod triage_config;
pub mod triage_service;
pub mod types;

pub use audit::{AuditRing, OpenOptions};
pub use classifier::classify;
pub use distro::{ArchTarget, CLibrary, DistroEvaluation, DistroFamily, DistroProfile, InitSystem};
pub use distro_service::DistroStore;
pub use handoff::{HandoffPriority, HandoffRecord, HandoffReport, HandoffStatus};
pub use handoff_service::HandoffStore;
pub use pep::PepStore;
pub use types::GENESIS_HASH;
