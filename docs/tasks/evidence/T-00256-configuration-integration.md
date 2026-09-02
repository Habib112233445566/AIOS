# T-00256 — Configuration: Integration
`release_config.load_config()` is called by `generate_release` and `physical_create_zip` on every invocation. Config is discoverable via `$AIOSH_RELEASE_CONFIG` env var. Cross-substrate: Rust module declared in `lib.rs`. Integration smoke: all 15 release tests pass (3 + 6 + 6).
