# T-00253 — Configuration: Scaffold
Created `aiosh_mcp/release_config.py` (Python) and `aiosh-core/src/release_config.rs` (Rust).
Both define `load_config()` with env var override and JSON file loading. Python tested: defaults load correctly. Rust module declared in `lib.rs`.
