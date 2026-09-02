# T-00321 — Dependency & Toolchain Pinning: core service Research

**Date:** 2026-08-27
**Type:** research (no code changed)
**Depends on:** T-00320 (complete)
**Artifact note:** instructions specify `T-00321-research.md`; the ledger row's `artifacts` field declares `T-00321-core-service-research.md` (mirrored byte-for-byte so the declared artifact exists).

---

## 1. What "core service" means for Toolchain Pinning

The data model step (T-00311..T-00320) shipped `aiosh-core/src/toolchain_config.rs`, which parses a `ToolchainManifest` containing `rust_version`, `python_version`, `node_version`, and `enforce_hashes`. The data model specifies *what* the rules are, but does not enforce them.

The **core service** for Dependency & Toolchain Pinning must provide the mechanism to query the host ecosystem (`rustc`, `python`, `node`), extract their active versions, compare them to the loaded `ToolchainManifest`, and enforce the pin.

## 2. Internal facts (read from the tree, 2026-08-27)

| # | Fact | Anchor |
|---|---|---|
| F1 | `ToolchainManifest` loads required versions from `$AIOSH_TOOLCHAIN_CONFIG` (or `config/toolchain.json`). | `aiosh-core/src/toolchain_config.rs` |
| F2 | The data model specification explicitly states: "Ecosystem integration (actually running `python --version` or invoking `cargo`) belongs in the subsequent Core Service components." | `docs/tasks/evidence/T-00312-data-model-specification.md` |
| F3 | `aiosh-core` does not currently invoke `rustc`, `python`, or `node` to check their versions. | `aiosh-core/src/` |
| F4 | Process invocation and capturing stdout/stderr is already patterned in `pentest.rs` and `release.rs` using `std::process::Command`. | `aiosh-core/src/release.rs`, `pentest.rs` |

## 3. External authoritative facts

Source: Standard CLI documentation for Rust, Python, and Node.js.

| # | Fact |
|---|---|
| E1 | **Rust:** `rustc -V` outputs the version in the format `rustc 1.80.0 (hash date)`. `cargo -V` outputs similarly. |
| E2 | **Python:** `python3 -V` or `python --version` outputs the version in the format `Python 3.10.12`. |
| E3 | **Node:** `node -v` or `node --version` outputs the version with a `v` prefix, e.g., `v20.10.0`. |
| E4 | **Rust `std::process::Command`:** Can be used to spawn these processes and capture `stdout`. Capturing `stdout` as UTF-8 allows string matching to verify the active version. |

## 4. Gap analysis

Currently, a developer or agent could invoke `aiosh` tasks or build processes in an environment with the wrong Rust or Python version, leading to non-reproducible outputs or poisoned builds. While `ToolchainManifest` can be loaded, there is no service to enforce it.

### Candidates considered

| Candidate | Verdict | Reasoning |
|---|---|---|
| **A. `aiosh_core::toolchain` service module** | **Recommended proposal** | A dedicated service module in `aiosh-core` that exposes an `enforce_toolchain(manifest: &ToolchainManifest) -> Result<(), String>` function. This function shells out to `rustc`, `python`, and `node`, parses their versions, and returns an error if they do not match the manifest. |
| B. Inline checks within `release.rs` or `task_service.rs` | Rejected | Duplicates logic if multiple subsystems need to verify the environment. Better to keep it as a standalone core service module. |

## 5. Assumptions (clearly marked, not facts)

- A1 (assumption): The core service will do string prefix matching or semver parsing. Exact string matching of the version number is the safest and most reproducible approach.
- A2 (assumption): If `node_version` is `None` in the manifest, the core service should skip checking Node.js.

## 6. Decisions needed before Specification (T-00322)

- **D1 — Parsing Strictness:** Do we require exact string equality (e.g. `rust_version: "1.80.0"` must exactly match the parsed `1.80.0` from `rustc -V`) or do we support semantic versioning requirements (e.g. `^1.80.0`)? (Recommendation: Exact match for pinning).
- **D2 — Missing Binaries:** If a binary (e.g. `python`) is entirely missing from `$PATH`, should the service return a specific error type, or just a generic string failure?
- **D3 — Integration Point:** Will this core service be invoked during `aiosh` CLI startup, or only lazily before a `release` or `task` execution? (This will determine if we need to cache the check result).

## 7. Acceptance check

- [x] Evidence file separates facts from assumptions and proposals.
- [x] Citations given for external sources.
- [x] Unknowns and decisions needed listed explicitly.
- [x] No code changed in this task.
