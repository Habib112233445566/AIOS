# T-00311 — Dependency & Toolchain Pinning: Data Model Research

## 1. Goal
Establish facts, constraints, and prior art for the data model of Dependency & Toolchain Pinning within the AIOS project. The objective is to guarantee deterministic, reproducible, and secure builds, preventing supply chain attacks and environmental drift.

## 2. Current State (Facts vs Assumptions)

### Facts (Observed in the Repository)
- **Rust Toolchain**: `code/aiosh-rust/Cargo.toml` uses loose version requirements (e.g., `serde = { version = "1" }`). Exact dependencies are pinned via `Cargo.lock`. However, there is no `rust-toolchain.toml` or `rust-version` field, meaning the Rust compiler version is completely unpinned and depends on the host's active `rustup` default.
- **Python Toolchain**: `code/aiosh-mcp/pyproject.toml` uses loose lower-bounds (`requires-python = ">=3.10"`, `fastmcp>=0.4.0`). There is no strict lockfile (e.g., `requirements.txt` with hashes, `poetry.lock`, or `uv.lock`) in the tree to guarantee determinism.
- **Node.js Toolchain**: `code/aiosh-cli/package.json` exists but does not pin the Node.js runtime version (e.g., via `engines` or `.nvmrc`).
- **OS/System Dependencies**: CI scripts (`ci/run_all_smokes.sh`) implicitly assume host presence of `bash`, `wsl.exe`, `genisoimage`, and `python3`, leading to environment-specific failures (e.g., `rust_smoke` failing on Windows without WSL).

### Assumptions (To be Resolved)
- *Assumption*: Native lockfiles (`Cargo.lock`, `package-lock.json`) are sufficient for dependency pinning. (False: They do not pin the compiler/runtime versions or OS-level binaries).
- *Assumption*: CI runners provide a stable environment. (False: Runner image updates frequently break loose bounds).

## 3. Prior Art & Authoritative Sources
1. **Rust / Cargo (RFC 2956)**: The canonical approach to pinning the Rust compiler and components is `rust-toolchain.toml`. This ensures all developers and CI runners use the exact same `rustc`, `clippy`, and `rustfmt` versions.
   - *Citation*: [The rustup book - The toolchain file](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)
2. **Python (PEP 665 / PEP 735)**: Python ecosystem standards advocate for deterministic lockfiles containing cryptographic hashes to prevent tampering. Modern toolchains (like `uv` or `pip-tools`) emit standard `requirements.txt` with `--require-hashes`.
   - *Citation*: [PEP 665 – A file format to list Python dependencies](https://peps.python.org/pep-0665/)
3. **Reproducible Builds**: The overarching industry standard for security-critical OS development requires bit-for-bit identical outputs. This demands pinning not just packages, but the entire build toolchain.
   - *Citation*: [reproducible-builds.org](https://reproducible-builds.org/)

## 4. Unknowns and Decisions Needed (Pre-Implementation)
Before moving to the Specification and Implementation tasks, the following decisions must be made:

1. **Toolchain Pinning Strategy**:
   - *Decision*: Do we rely on ecosystem-specific files (`rust-toolchain.toml`, `.nvmrc`, `.python-version`) or build a unified `aios-toolchain.json` that our own system enforces?
   - *Recommendation*: Ecosystem-specific files are natively supported by `cargo`, `nvm`, and `pyenv`/`uv`, reducing friction. We should use `rust-toolchain.toml`.
2. **Python Strictness**:
   - *Decision*: How should we lock the Python dependencies for `aiosh-mcp`?
   - *Recommendation*: Introduce a strict `requirements-locked.txt` generated with cryptographic hashes to ensure `mcp` and `fastmcp` dependencies cannot be poisoned upstream.
3. **OS-Level Pinning**:
   - *Decision*: How do we pin `genisoimage` and the bash execution environment? 
   - *Recommendation*: We need a data model to declare system-level prerequisites (either via a `Dockerfile`, a Nix `flake.nix`, or a native AIOS dependency declaration).

## 5. Conclusion
To satisfy the security and reproducibility pillars of AIOS, our data model must evolve from loose semantic versioning to strict cryptographic hashes (`Cargo.lock`, `requirements-locked.txt`) combined with strict runtime version pinning (`rust-toolchain.toml`, `.python-version`).
