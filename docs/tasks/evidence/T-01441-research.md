# T-01441: User Session Bootstrap — Configuration: Research

See detailed evidence in [T-01441-configuration-research.md](T-01441-configuration-research.md).

## Summary
- Established authoritative prior art from `logind.conf(5)`, `pam_limits(8)`, and XDG Base Directory specification.
- Evaluated existing configuration modules in `code/aiosh-rust/aiosh-core/src/service_config.rs` and `package_config.rs`.
- Separated empirical facts from operational assumptions.
- Codified decisions on paths (`.aios/session_store.json`, `config/session.json`), env prefix (`AIOS_SESSION_*`), and configuration invariants `SC1..SC7`.
